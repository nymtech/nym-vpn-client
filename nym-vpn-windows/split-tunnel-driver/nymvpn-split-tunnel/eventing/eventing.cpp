// Copyright 2016-2026 Mullvad VPN AB. All Rights Reserved.
// Copyright 2026 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

#include "eventing.h"
#include "context.h"
#include "builder.h"
#include "../defs/types.h"

#include "../trace.h"
#include "eventing.tmh"

namespace eventing {

namespace {

void EnqueueEvent(CONTEXT* Context, RAW_EVENT* evt) {
    WdfSpinLockAcquire(Context->EventQueueLock);

    const SIZE_T MAX_QUEUED_EVENTS = 100;

    //
    // Discard oldest events if events are too numerous.
    //

    while (Context->NumEvents >= MAX_QUEUED_EVENTS) {
        auto oldEvent = (RAW_EVENT*)RemoveHeadList(&Context->EventQueue);

        --Context->NumEvents;

        ReleaseEvent(&oldEvent);
    }

    //
    // Add new event at end of queue.
    //

    InsertTailList(&Context->EventQueue, &evt->ListEntry);

    ++Context->NumEvents;

    WdfSpinLockRelease(Context->EventQueueLock);
}

void CompleteRequestReleaseEvent(WDFREQUEST Request, void* RequestBuffer, RAW_EVENT* Event) {
    RtlCopyMemory(RequestBuffer, Event->Buffer, Event->BufferSize);

    WdfRequestCompleteWithInformation(Request, STATUS_SUCCESS, Event->BufferSize);

    ReleaseEvent(&Event);
}

} // anonymous namespace

NTSTATUS
Initialize(CONTEXT** Context, WDFDEVICE Device) {
    *Context = NULL;

    auto context = (CONTEXT*)ExAllocatePoolUninitialized(NonPagedPool, sizeof(CONTEXT), ST_POOL_TAG);

    if (context == NULL) {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    RtlZeroMemory(context, sizeof(*context));

    InitializeListHead(&context->EventQueue);

    auto status = WdfSpinLockCreate(WDF_NO_OBJECT_ATTRIBUTES, &context->EventQueueLock);

    if (!NT_SUCCESS(status)) {
        DbgPrint("WdfSpinLockCreate() failed 0x%X\n", status);

        goto Abort;
    }

    WDF_IO_QUEUE_CONFIG queueConfig;

    WDF_IO_QUEUE_CONFIG_INIT(&queueConfig, WdfIoQueueDispatchManual);

    queueConfig.PowerManaged = WdfFalse;

    status = WdfIoQueueCreate(Device, &queueConfig, WDF_NO_OBJECT_ATTRIBUTES, &context->RequestQueue);

    if (!NT_SUCCESS(status)) {
        DbgPrint("WdfIoQueueCreate() failed 0x%X\n", status);

        goto Abort_delete_lock;
    }

    *Context = context;

    return STATUS_SUCCESS;

Abort_delete_lock:

    WdfObjectDelete(context->EventQueueLock);

Abort:

    ExFreePoolWithTag(context, ST_POOL_TAG);

    return status;
}

void TearDown(CONTEXT** Context) {
    auto context = *Context;

    //
    // Discard and release all queued events.
    // Don't use the lock because if there's contension we've already failed.
    //

    while (!IsListEmpty(&context->EventQueue)) {
        auto evt = (RAW_EVENT*)RemoveHeadList(&context->EventQueue);

        ReleaseEvent(&evt);
    }

    context->NumEvents = 0;

    //
    // Cancel all queued requests.
    //

    WDFREQUEST pendedRequest;

    for (;;) {
        auto status = WdfIoQueueRetrieveNextRequest(context->RequestQueue, &pendedRequest);

        if (!NT_SUCCESS(status) || pendedRequest == NULL) {
            break;
        }

        WdfRequestComplete(pendedRequest, STATUS_CANCELLED);
    }

    //
    // Delete all objects.
    //

    WdfObjectDelete(context->RequestQueue);
    WdfObjectDelete(context->EventQueueLock);

    //
    // Release context.
    //

    ExFreePoolWithTag(context, ST_POOL_TAG);

    *Context = NULL;
}

void Emit(CONTEXT* Context, RAW_EVENT** Event) {
    auto evt = *Event;

    if (evt == NULL) {
        return;
    }

    *Event = NULL;

    WDFREQUEST pendedRequest;

    void* buffer;

    //
    // Look for a pended request with a correctly sized buffer.
    //
    // Fail all requests we encounter that have tiny buffers.
    // User mode should know better.
    //

    for (;;) {
        auto status = WdfIoQueueRetrieveNextRequest(Context->RequestQueue, &pendedRequest);

        if (!NT_SUCCESS(status) || pendedRequest == NULL) {
            EnqueueEvent(Context, evt);

            return;
        }

        status = WdfRequestRetrieveOutputBuffer(pendedRequest, evt->BufferSize, &buffer, NULL);

        if (NT_SUCCESS(status)) {
            break;
        }

        WdfRequestComplete(pendedRequest, status);
    }

    CompleteRequestReleaseEvent(pendedRequest, buffer, evt);
}

void CollectOne(CONTEXT* Context, WDFREQUEST Request) {
    RAW_EVENT* evt = NULL;

    WdfSpinLockAcquire(Context->EventQueueLock);

    if (!IsListEmpty(&Context->EventQueue)) {
        evt = (RAW_EVENT*)RemoveHeadList(&Context->EventQueue);

        --Context->NumEvents;
    }

    WdfSpinLockRelease(Context->EventQueueLock);

    if (evt == NULL) {
        auto status = WdfRequestForwardToIoQueue(Request, Context->RequestQueue);

        if (!NT_SUCCESS(status)) {
            DbgPrint("Failed to pend event request\n");

            WdfRequestComplete(Request, STATUS_INTERNAL_ERROR);
        }

        return;
    }

    //
    // Acquire and validate request buffer.
    //

    void* buffer;

    auto status = WdfRequestRetrieveOutputBuffer(Request, evt->BufferSize, &buffer, NULL);

    if (!NT_SUCCESS(status)) {
        WdfRequestComplete(Request, status);

        //
        // Put the event back.
        //

        WdfSpinLockAcquire(Context->EventQueueLock);

        InsertHeadList(&Context->EventQueue, &evt->ListEntry);

        ++Context->NumEvents;

        WdfSpinLockRelease(Context->EventQueueLock);

        return;
    }

    CompleteRequestReleaseEvent(Request, buffer, evt);
}

} // namespace eventing
