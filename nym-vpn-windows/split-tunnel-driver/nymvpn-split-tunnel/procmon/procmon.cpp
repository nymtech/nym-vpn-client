// Copyright 2016-2026 Mullvad VPN AB. All Rights Reserved.
// Copyright 2026 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

#include <ntddk.h>
#include "procmon.h"
#include "context.h"
#include "../util.h"
#include "../defs/types.h"

#include "../trace.h"
#include "procmon.tmh"

namespace procmon {

namespace {

//
// PsSetCreateProcessNotifyRoutineEx() is broken so you can't pass context.
//
// This isn't ideal, especially considering creating more than once instance of this "class" will
// send all events to the most recently registered sink.
//
// But... There should never be more than one instance.
// And this lets us keep a familiar interface towards clients, so just roll with it.
//
CONTEXT* g_Context = NULL;

void SystemProcessEvent(PEPROCESS Process, HANDLE ProcessId, PPS_CREATE_NOTIFY_INFO CreateInfo) {
    //
    // We want to offload the system thread this is being sent on.
    // Build a self-contained event record and queue it to a dedicated thread.
    //

    PROCESS_EVENT* record = NULL;

    if (CreateInfo != NULL) {
        //
        // Process is arriving.
        //
        // First, get the filename so we can determine the size of the final
        // buffer that needs to be allocated.
        //

        UNICODE_STRING* imageName;

        auto status = util::GetDevicePathImageName(Process, &imageName);

        if (!NT_SUCCESS(status)) {
            DbgPrint("Dropping process event\n");
            DbgPrint("  Could not determine image filename, status: 0x%X\n", status);
            DbgPrint("  PID of arriving process %p\n", ProcessId);

            return;
        }

        auto offsetDetails = util::RoundToMultiple(sizeof(PROCESS_EVENT), TYPE_ALIGNMENT(PROCESS_EVENT_DETAILS));
        auto offsetStringBuffer =
            util::RoundToMultiple(offsetDetails + sizeof(PROCESS_EVENT_DETAILS), TYPE_ALIGNMENT(WCHAR));

        auto allocationSize = offsetStringBuffer + imageName->Length;

        record = (PROCESS_EVENT*)ExAllocatePoolUninitialized(PagedPool, allocationSize, ST_POOL_TAG);

        if (record == NULL) {
            DbgPrint("Dropping process event\n");
            DbgPrint("  Failed to allocate memory\n");
            DbgPrint("  Imagename of arriving process %wZ\n", imageName);
            DbgPrint("  PID of arriving process %p\n", ProcessId);

            ExFreePoolWithTag(imageName, ST_POOL_TAG);

            return;
        }

        auto details = (PROCESS_EVENT_DETAILS*)(((CHAR*)record) + offsetDetails);
        auto stringBuffer = (WCHAR*)(((CHAR*)record) + offsetStringBuffer);

        InitializeListHead(&record->ListEntry);
        record->ProcessId = ProcessId;
        record->Details = details;

        details->ParentProcessId = CreateInfo->ParentProcessId;
        details->ImageName.Length = imageName->Length;
        details->ImageName.MaximumLength = imageName->Length;
        details->ImageName.Buffer = stringBuffer;

        RtlCopyMemory(stringBuffer, imageName->Buffer, imageName->Length);
        ExFreePoolWithTag(imageName, ST_POOL_TAG);
    } else {
        //
        // Process is departing.
        //

        record = (PROCESS_EVENT*)ExAllocatePoolUninitialized(PagedPool, sizeof(PROCESS_EVENT), ST_POOL_TAG);

        if (record == NULL) {
            DbgPrint("Dropping process event\n");
            DbgPrint("  Failed to allocate memory\n");
            DbgPrint("  PID of departing process %p\n", ProcessId);

            return;
        }

        InitializeListHead(&record->ListEntry);
        record->ProcessId = ProcessId;
        record->Details = NULL;
    }

    //
    // Queue to worker thread.
    //

    WdfWaitLockAcquire(g_Context->QueueLock, NULL);

    InsertTailList(&g_Context->EventQueue, &record->ListEntry);

    if (g_Context->DispatchingEnabled) {
        KeSetEvent(&g_Context->WakeUpWorker, 0, FALSE);
    }

    WdfWaitLockRelease(g_Context->QueueLock);
}

void DispatchWorker(PVOID StartContext) {
    auto context = (CONTEXT*)StartContext;

    for (;;) {
        KeWaitForSingleObject(&context->WakeUpWorker, Executive, KernelMode, FALSE, NULL);

        WdfWaitLockAcquire(context->QueueLock, NULL);

        if (0 != KeReadStateEvent(&context->ExitWorker)) {
            WdfWaitLockRelease(context->QueueLock);

            PsTerminateSystemThread(STATUS_SUCCESS);

            return;
        }

        //
        // Reparent the queue in order to release the lock sooner.
        //

        LIST_ENTRY queue;

        util::ReparentList(&queue, &context->EventQueue);

        KeClearEvent(&context->WakeUpWorker);

        WdfWaitLockRelease(context->QueueLock);

        //
        // There are one or more records queued.
        // Process all available records.
        //

        LIST_ENTRY* record;

        while ((record = RemoveHeadList(&queue)) != &queue) {
            context->ProcessEventSink((PROCESS_EVENT*)record, context->SinkContext);

            ExFreePoolWithTag(record, ST_POOL_TAG);
        }
    }
}

} // anonymous namespace

NTSTATUS
Initialize(CONTEXT** Context, PROCESS_EVENT_SINK ProcessEventSink, void* SinkContext) {
    *Context = NULL;

    bool notifyRoutineRegistered = false;

    auto context = (CONTEXT*)ExAllocatePoolUninitialized(NonPagedPool, sizeof(CONTEXT), ST_POOL_TAG);

    if (context == NULL) {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    RtlZeroMemory(context, sizeof(*context));

    context->ProcessEventSink = ProcessEventSink;
    context->SinkContext = SinkContext;

    InitializeListHead(&context->EventQueue);

    KeInitializeEvent(&context->ExitWorker, NotificationEvent, FALSE);
    KeInitializeEvent(&context->WakeUpWorker, NotificationEvent, FALSE);

    auto status = WdfWaitLockCreate(WDF_NO_OBJECT_ATTRIBUTES, &context->QueueLock);

    if (!NT_SUCCESS(status)) {
        DbgPrint("WdfWaitLockCreate() failed 0x%X\n", status);

        context->QueueLock = NULL;

        goto Abort;
    }

    g_Context = context;

    //
    // It's alright to register for notifications before starting the worker thread.
    //
    // Events that come in before the thread is created are queued.
    // So no event will be lost.
    //
    // Also, the thread doesn't own the queued events so nothing is leaked even
    // if the thread fails to process events in a timely manner, or at all.
    //
    // Also, clean-up is simpler if thread creation is the last fallible operation.
    //

    status = PsSetCreateProcessNotifyRoutineEx(SystemProcessEvent, FALSE);

    if (!NT_SUCCESS(status)) {
        DbgPrint("PsSetCreateProcessNotifyRoutineEx() failed 0x%X\n", status);

        goto Abort;
    }

    notifyRoutineRegistered = true;

    //
    // Create the thread that will be servicing events.
    //

    OBJECT_ATTRIBUTES threadAttributes;

    InitializeObjectAttributes(&threadAttributes, NULL, OBJ_KERNEL_HANDLE, NULL, NULL);

    HANDLE threadHandle;

    status =
        PsCreateSystemThread(&threadHandle, THREAD_ALL_ACCESS, &threadAttributes, NULL, NULL, DispatchWorker, context);

    if (!NT_SUCCESS(status)) {
        DbgPrint("PsCreateSystemThread() failed 0x%X\n", status);
        DbgPrint("Could not create process monitoring thread\n");

        goto Abort;
    }

    //
    // ObReference... will never fail if the handle is valid.
    //

    status = ObReferenceObjectByHandle(
        threadHandle, THREAD_ALL_ACCESS, NULL, KernelMode, (PVOID*)&context->DispatchWorker, NULL);

    ZwClose(threadHandle);

    *Context = context;

    return STATUS_SUCCESS;

Abort:

    if (notifyRoutineRegistered) {
        PsSetCreateProcessNotifyRoutineEx(SystemProcessEvent, TRUE);

        //
        // Drain event queue to avoid leaking events.
        //

        LIST_ENTRY* record;

        while ((record = RemoveHeadList(&context->EventQueue)) != &context->EventQueue) {
            ExFreePoolWithTag(record, ST_POOL_TAG);
        }
    }

    if (context->QueueLock != NULL) {
        WdfObjectDelete(context->QueueLock);
    }

    ExFreePoolWithTag(context, ST_POOL_TAG);
    g_Context = NULL;

    return status;
}

void TearDown(CONTEXT** Context) {
    auto context = *Context;

    //
    // Deregister notify routine so we stop queuing events.
    // This can never fail according to documentation.
    //

    PsSetCreateProcessNotifyRoutineEx(SystemProcessEvent, TRUE);

    //
    // Tell worker thread to exit and wait for it to happen.
    //

    WdfWaitLockAcquire(context->QueueLock, NULL);

    KeSetEvent(&context->ExitWorker, 0, FALSE);
    KeSetEvent(&context->WakeUpWorker, 1, FALSE);

    WdfWaitLockRelease(context->QueueLock);

    KeWaitForSingleObject(context->DispatchWorker, Executive, KernelMode, FALSE, NULL);

    ObDereferenceObject(context->DispatchWorker);

    //
    // Drain event queue to avoid leaking events.
    //

    LIST_ENTRY* record;

    while ((record = RemoveHeadList(&context->EventQueue)) != &context->EventQueue) {
        ExFreePoolWithTag(record, ST_POOL_TAG);
    }

    //
    // Release remaining resources.
    //

    WdfObjectDelete(context->QueueLock);

    ExFreePoolWithTag(context, ST_POOL_TAG);

    *Context = NULL;
}

void EnableDispatching(CONTEXT* Context) {
    WdfWaitLockAcquire(Context->QueueLock, NULL);

    Context->DispatchingEnabled = true;

    if (!IsListEmpty(&Context->EventQueue)) {
        KeSetEvent(&Context->WakeUpWorker, 0, FALSE);
    }

    WdfWaitLockRelease(Context->QueueLock);
}

} // namespace procmon
