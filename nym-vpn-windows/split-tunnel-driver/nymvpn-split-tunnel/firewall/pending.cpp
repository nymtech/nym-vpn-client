// Copyright 2016-2026 Mullvad VPN AB. All Rights Reserved.
// Copyright 2026 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

#include "pending.h"
#include "classify.h"
#include "../util.h"

#include "../trace.h"
#include "pending.tmh"

namespace firewall { namespace pending {

struct PENDED_CLASSIFICATION {
    LIST_ENTRY ListEntry;

    // Process that's making the request.
    HANDLE ProcessId;

    // Timestamp when record was created.
    ULONGLONG Timestamp;

    // Handle used to trigger re-auth or resume processing.
    UINT64 ClassifyHandle;

    // Result of classification is recorded here.
    FWPS_CLASSIFY_OUT0 ClassifyOut;

    // Filter that triggered the classification.
    UINT64 FilterId;

    // Layer in which classification is occurring.
    UINT16 LayerId;
};

struct CONTEXT {
    procbroker::CONTEXT* ProcessEventBroker;

    WDFSPINLOCK Lock;

    // PENDED_CLASSIFICATION
    LIST_ENTRY Classifications;
};

namespace {

const ULONGLONG RECORD_MAX_LIFETIME_MS = 10000;

bool AssertCompatibleLayer(UINT16 LayerId) {
    NT_ASSERT(
        LayerId == FWPS_LAYER_ALE_BIND_REDIRECT_V4 || LayerId == FWPS_LAYER_ALE_BIND_REDIRECT_V6 ||
        LayerId == FWPS_LAYER_ALE_CONNECT_REDIRECT_V4 || LayerId == FWPS_LAYER_ALE_CONNECT_REDIRECT_V6);

    if (LayerId == FWPS_LAYER_ALE_BIND_REDIRECT_V4 || LayerId == FWPS_LAYER_ALE_BIND_REDIRECT_V6 ||
        LayerId == FWPS_LAYER_ALE_CONNECT_REDIRECT_V4 || LayerId == FWPS_LAYER_ALE_CONNECT_REDIRECT_V6) {
        return true;
    }

    DbgPrint("Invalid layer id %d specified in call to 'pending' module\n", LayerId);

    return false;
}

char const* LayerToString(UINT16 LayerId) {
    char const* s = "undefined";

    switch (LayerId) {
    case FWPS_LAYER_ALE_BIND_REDIRECT_V4: {
        s = "FWPS_LAYER_ALE_BIND_REDIRECT_V4";
        break;
    }
    case FWPS_LAYER_ALE_BIND_REDIRECT_V6: {
        s = "FWPS_LAYER_ALE_BIND_REDIRECT_V6";
        break;
    }
    case FWPS_LAYER_ALE_CONNECT_REDIRECT_V4: {
        s = "FWPS_LAYER_ALE_CONNECT_REDIRECT_V4";
        break;
    }
    case FWPS_LAYER_ALE_CONNECT_REDIRECT_V6: {
        s = "FWPS_LAYER_ALE_CONNECT_REDIRECT_V6";
        break;
    }
    }

    return s;
}

NTSTATUS
FailRequest(UINT64 FilterId, UINT16 LayerId, FWPS_CLASSIFY_OUT0* ClassifyOut, UINT64 ClassifyHandle) {
    //
    // There doesn't seem to be any support in WFP for blocking requests in the redirect layers.
    // Specifying `FWP_ACTION_BLOCK` will just resume request processing.
    // So the best we can do is rewrite the request to do as little harm as possible.
    //

    PVOID requestData = NULL;

    auto status = FwpsAcquireWritableLayerDataPointer0(ClassifyHandle, FilterId, 0, &requestData, ClassifyOut);

    if (!NT_SUCCESS(status)) {
        DbgPrint("FwpsAcquireWritableLayerDataPointer0() failed\n");

        return status;
    }

    switch (LayerId) {
    case FWPS_LAYER_ALE_BIND_REDIRECT_V4:
    case FWPS_LAYER_ALE_BIND_REDIRECT_V6: {
        auto bindRequest = reinterpret_cast<FWPS_BIND_REQUEST0*>(requestData);

        //
        // This sets the port to 0, as well.
        //
        INETADDR_SETLOOPBACK((PSOCKADDR) & (bindRequest->localAddressAndPort));

        break;
    }
    case FWPS_LAYER_ALE_CONNECT_REDIRECT_V4:
    case FWPS_LAYER_ALE_CONNECT_REDIRECT_V6: {
        auto connectRequest = reinterpret_cast<FWPS_CONNECT_REQUEST0*>(requestData);

        INETADDR_SETLOOPBACK((PSOCKADDR) & (connectRequest->localAddressAndPort));

        break;
    }
    };

    ClassificationApplyHardPermit(ClassifyOut);
    FwpsApplyModifiedLayerData0(ClassifyHandle, requestData, 0);

    return STATUS_SUCCESS;
}

void ReauthPendedRequest(PENDED_CLASSIFICATION* Record) {
    DbgPrint(
        "Requesting re-auth for pended request in layer %s for process %p\n",
        LayerToString(Record->LayerId),
        Record->ProcessId);

    FwpsCompleteClassify0(Record->ClassifyHandle, 0, NULL);
    FwpsReleaseClassifyHandle0(Record->ClassifyHandle);

    ExFreePoolWithTag(Record, ST_POOL_TAG);
}

void FailPendedRequest(PENDED_CLASSIFICATION* Record, bool ReauthOnFailure = true) {
    DbgPrint("Failing pended request in layer %s for process %p\n", LayerToString(Record->LayerId), Record->ProcessId);

    auto const status = FailRequest(Record->FilterId, Record->LayerId, &Record->ClassifyOut, Record->ClassifyHandle);

    if (NT_SUCCESS(status)) {
        FwpsCompleteClassify0(Record->ClassifyHandle, 0, &Record->ClassifyOut);
        FwpsReleaseClassifyHandle0(Record->ClassifyHandle);
    } else {
        DbgPrint("FailRequest() failed 0x%X\n", status);

        if (ReauthOnFailure) {
            ReauthPendedRequest(Record);

            return;
        }
    }

    ExFreePoolWithTag(Record, ST_POOL_TAG);
}

//
// FailAllPendedRequests()
//
// This function is used during tear down.
// So we don't have the luxury of re-authing requests that can't be failed.
//
void FailAllPendedRequests(CONTEXT* Context) {
    for (auto rawRecord = Context->Classifications.Flink; rawRecord != &Context->Classifications;
         /* no post-condition */) {
        auto record = (PENDED_CLASSIFICATION*)rawRecord;

        rawRecord = rawRecord->Flink;

        RemoveEntryList(&record->ListEntry);

        FailPendedRequest(record, false);
    }
}

void HandleProcessEvent(HANDLE ProcessId, bool Arriving, void* Context) {
    auto context = (CONTEXT*)Context;

    auto timeNow = KeQueryInterruptTime();

    static const ULONGLONG MS_TO_100NS_FACTOR = 10000;

    auto maxAge = RECORD_MAX_LIFETIME_MS * MS_TO_100NS_FACTOR;

    //
    // Collect records to process while holding the lock, then process them
    // after releasing it.
    //
    // This is required because completing or re-authing a pended classification
    // (FwpsCompleteClassify0) can synchronously re-invoke a callout, which will
    // attempt to acquire this same lock - causing a deadlock spin at DISPATCH_LEVEL
    // that starves the scheduler.
    //

    LIST_ENTRY toReauth;
    LIST_ENTRY toFail;
    LIST_ENTRY toFailNoReauth;

    InitializeListHead(&toReauth);
    InitializeListHead(&toFail);
    InitializeListHead(&toFailNoReauth);

    WdfSpinLockAcquire(context->Lock);

    for (auto rawRecord = context->Classifications.Flink; rawRecord != &context->Classifications;
         /* no post-condition */) {
        auto record = (PENDED_CLASSIFICATION*)rawRecord;

        rawRecord = rawRecord->Flink;

        auto timeDelta = timeNow - record->Timestamp;

        if (timeDelta > maxAge) {
            RemoveEntryList(&record->ListEntry);
            InsertTailList(&toFail, &record->ListEntry);
            continue;
        }

        if (record->ProcessId != ProcessId) {
            continue;
        }

        RemoveEntryList(&record->ListEntry);

        if (Arriving) {
            InsertTailList(&toReauth, &record->ListEntry);
        } else {
            InsertTailList(&toFailNoReauth, &record->ListEntry);
        }
    }

    WdfSpinLockRelease(context->Lock);

    for (auto rawRecord = toReauth.Flink; rawRecord != &toReauth; /* no post-condition */) {
        auto record = (PENDED_CLASSIFICATION*)rawRecord;
        rawRecord = rawRecord->Flink;
        ReauthPendedRequest(record);
    }

    for (auto rawRecord = toFail.Flink; rawRecord != &toFail; /* no post-condition */) {
        auto record = (PENDED_CLASSIFICATION*)rawRecord;
        rawRecord = rawRecord->Flink;
        FailPendedRequest(record);
    }

    for (auto rawRecord = toFailNoReauth.Flink; rawRecord != &toFailNoReauth; /* no post-condition */) {
        auto record = (PENDED_CLASSIFICATION*)rawRecord;
        rawRecord = rawRecord->Flink;
        FailPendedRequest(record, false);
    }
}

} // anonymous namespace

NTSTATUS
Initialize(CONTEXT** Context, procbroker::CONTEXT* ProcessEventBroker) {
    auto context = (CONTEXT*)ExAllocatePoolUninitialized(NonPagedPool, sizeof(CONTEXT), ST_POOL_TAG);

    if (context == NULL) {
        DbgPrint("ExAllocatePoolUninitialized() failed\n");

        return STATUS_INSUFFICIENT_RESOURCES;
    }

    RtlZeroMemory(context, sizeof(*context));

    context->ProcessEventBroker = ProcessEventBroker;

    auto status = WdfSpinLockCreate(WDF_NO_OBJECT_ATTRIBUTES, &context->Lock);

    if (!NT_SUCCESS(status)) {
        DbgPrint("WdfSpinLockCreate() failed\n");

        goto Abort;
    }

    InitializeListHead(&context->Classifications);

    //
    // Everything is initialized.
    // Register with process event broker.
    //

    status = procbroker::Subscribe(ProcessEventBroker, HandleProcessEvent, context);

    if (!NT_SUCCESS(status)) {
        DbgPrint("Could not register with process event broker\n");

        goto Abort_delete_lock;
    }

    *Context = context;

    return STATUS_SUCCESS;

Abort_delete_lock:

    WdfObjectDelete(context->Lock);

Abort:

    ExFreePoolWithTag(context, ST_POOL_TAG);

    return status;
}

void TearDown(CONTEXT** Context) {
    auto context = *Context;

    *Context = NULL;

    procbroker::CancelSubscription(context->ProcessEventBroker, HandleProcessEvent);

    FailAllPendedRequests(context);

    WdfObjectDelete(context->Lock);

    ExFreePoolWithTag(context, ST_POOL_TAG);
}

NTSTATUS
PendRequest(
    CONTEXT* Context,
    HANDLE ProcessId,
    void* ClassifyContext,
    UINT64 FilterId,
    UINT16 LayerId,
    FWPS_CLASSIFY_OUT0* ClassifyOut) {
    if (!AssertCompatibleLayer(LayerId)) {
        return STATUS_UNSUCCESSFUL;
    }

    DbgPrint("Pending request in layer %s for process %p\n", LayerToString(LayerId), ProcessId);

    auto record =
        (PENDED_CLASSIFICATION*)ExAllocatePoolUninitialized(NonPagedPool, sizeof(PENDED_CLASSIFICATION), ST_POOL_TAG);

    if (record == NULL) {
        DbgPrint("ExAllocatePoolUninitialized() failed\n");

        return STATUS_INSUFFICIENT_RESOURCES;
    }

    UINT64 classifyHandle;

    auto status = FwpsAcquireClassifyHandle0(ClassifyContext, 0, &classifyHandle);

    if (!NT_SUCCESS(status)) {
        DbgPrint("FwpsAcquireClassifyHandle0() failed\n");

        goto Abort;
    }

    status = FwpsPendClassify0(classifyHandle, FilterId, 0, ClassifyOut);

    if (!NT_SUCCESS(status)) {
        DbgPrint("FwpsPendClassify0() failed\n");

        FwpsReleaseClassifyHandle0(classifyHandle);

        goto Abort;
    }

    record->ProcessId = ProcessId;
    record->Timestamp = KeQueryInterruptTime();
    record->ClassifyHandle = classifyHandle;
    record->ClassifyOut = *ClassifyOut;
    record->LayerId = LayerId;
    record->FilterId = FilterId;

    WdfSpinLockAcquire(Context->Lock);

    InsertTailList(&Context->Classifications, &record->ListEntry);

    WdfSpinLockRelease(Context->Lock);

    return STATUS_SUCCESS;

Abort:

    ExFreePoolWithTag(record, ST_POOL_TAG);

    return status;
}

NTSTATUS
FailRequest(HANDLE ProcessId, void* ClassifyContext, UINT64 FilterId, UINT16 LayerId, FWPS_CLASSIFY_OUT0* ClassifyOut) {
    if (!AssertCompatibleLayer(LayerId)) {
        return STATUS_UNSUCCESSFUL;
    }

    DbgPrint("Failing request in layer %s for process %p\n", LayerToString(LayerId), ProcessId);

    UINT64 classifyHandle = 0;

    auto status = FwpsAcquireClassifyHandle0(ClassifyContext, 0, &classifyHandle);

    if (!NT_SUCCESS(status)) {
        DbgPrint("FwpsAcquireClassifyHandle0() failed\n");

        return status;
    }

    status = FailRequest(FilterId, LayerId, ClassifyOut, classifyHandle);

    if (!NT_SUCCESS(status)) {
        return status;
    }

    FwpsReleaseClassifyHandle0(classifyHandle);

    return STATUS_SUCCESS;
}

}} // namespace firewall::pending
