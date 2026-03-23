// Copyright 2016-2026 Mullvad VPN AB. All Rights Reserved.
// Copyright 2026 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

#include "appfilters.h"
#include "../defs/types.h"
#include "../util.h"
#include "constants.h"
#include "identifiers.h"
#include "wfp.h"

#include "../trace.h"
#include "appfilters.tmh"

namespace firewall::appfilters {

namespace {

struct BLOCK_CONNECTIONS_ENTRY {
    LIST_ENTRY ListEntry;

    //
    // Device path using all lower-case characters.
    //
    LOWER_UNICODE_STRING ImageName;

    //
    // Number of process instances that use this entry.
    //
    SIZE_T RefCount;

    //
    // WFP filter IDs.
    //
    UINT64 OutboundFilterIdV4;
    UINT64 InboundFilterIdV4;
    UINT64 OutboundFilterIdV6;
    UINT64 InboundFilterIdV6;
};

struct APP_FILTERS_CONTEXT {
    HANDLE WfpSession;

    LIST_ENTRY BlockedTunnelConnections;

    LIST_ENTRY TransactionEvents;
};

//
// Transaction events represent logically atomic operations on the list of
// block-connection entries.
//
// The most recent event is represented by the transaction event record at the
// head of the transaction record list.
//
// An individual event record states what action needs to be taken
// to undo a change to the block-connection list.
//
enum class TRANSACTION_EVENT_TYPE {
    INCREMENT_REF_COUNT,
    DECREMENT_REF_COUNT,
    ADD_ENTRY,
    REMOVE_ENTRY,
    SWAP_LISTS
};

struct TRANSACTION_EVENT {
    LIST_ENTRY ListEntry;
    TRANSACTION_EVENT_TYPE EventType;
    BLOCK_CONNECTIONS_ENTRY* Target;
};

struct TRANSACTION_EVENT_ADD_ENTRY {
    LIST_ENTRY ListEntry;
    TRANSACTION_EVENT_TYPE EventType;
    BLOCK_CONNECTIONS_ENTRY* Target;

    //
    // This may or may not be the real list head.
    // We insert to the right of it.
    //
    LIST_ENTRY* MockHead;
};

struct TRANSACTION_EVENT_SWAP_LISTS {
    LIST_ENTRY ListEntry;
    TRANSACTION_EVENT_TYPE EventType;

    //
    // This is the list head of the previous list.
    //
    LIST_ENTRY BlockedTunnelConnections;
};

NTSTATUS
PushTransactionEvent(LIST_ENTRY* TransactionEvents, TRANSACTION_EVENT_TYPE EventType, BLOCK_CONNECTIONS_ENTRY* Target) {
    auto evt = (TRANSACTION_EVENT*)ExAllocatePoolUninitialized(NonPagedPool, sizeof(TRANSACTION_EVENT), ST_POOL_TAG);

    if (evt == NULL) {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    InitializeListHead(&evt->ListEntry);

    evt->EventType = EventType;
    evt->Target = Target;

    InsertHeadList(TransactionEvents, &evt->ListEntry);

    return STATUS_SUCCESS;
}

NTSTATUS
TransactionIncrementedRefCount(LIST_ENTRY* TransactionEvents, BLOCK_CONNECTIONS_ENTRY* Target) {
    return PushTransactionEvent(TransactionEvents, TRANSACTION_EVENT_TYPE::DECREMENT_REF_COUNT, Target);
}

NTSTATUS
TransactionDecrementedRefCount(LIST_ENTRY* TransactionEvents, BLOCK_CONNECTIONS_ENTRY* Target) {
    return PushTransactionEvent(TransactionEvents, TRANSACTION_EVENT_TYPE::INCREMENT_REF_COUNT, Target);
}

NTSTATUS
TransactionAddedEntry(LIST_ENTRY* TransactionEvents, BLOCK_CONNECTIONS_ENTRY* Target) {
    return PushTransactionEvent(TransactionEvents, TRANSACTION_EVENT_TYPE::REMOVE_ENTRY, Target);
}

NTSTATUS
TransactionRemovedEntry(LIST_ENTRY* TransactionEvents, BLOCK_CONNECTIONS_ENTRY* Target, LIST_ENTRY* MockHead) {
    auto evt = (TRANSACTION_EVENT_ADD_ENTRY*)ExAllocatePoolUninitialized(
        NonPagedPool, sizeof(TRANSACTION_EVENT_ADD_ENTRY), ST_POOL_TAG);

    if (evt == NULL) {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    InitializeListHead(&evt->ListEntry);

    evt->EventType = TRANSACTION_EVENT_TYPE::ADD_ENTRY;
    evt->Target = Target;
    evt->MockHead = MockHead;

    InsertHeadList(TransactionEvents, &evt->ListEntry);

    return STATUS_SUCCESS;
}

NTSTATUS
TransactionSwappedLists(LIST_ENTRY* TransactionEvents, LIST_ENTRY* BlockedTunnelConnections) {
    auto evt = (TRANSACTION_EVENT_SWAP_LISTS*)ExAllocatePoolUninitialized(
        NonPagedPool, sizeof(TRANSACTION_EVENT_SWAP_LISTS), ST_POOL_TAG);

    if (evt == NULL) {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    InitializeListHead(&evt->ListEntry);

    evt->EventType = TRANSACTION_EVENT_TYPE::SWAP_LISTS;

    //
    // Ownership of list is moved to transaction entry.
    //

    util::ReparentList(&evt->BlockedTunnelConnections, BlockedTunnelConnections);

    InsertHeadList(TransactionEvents, &evt->ListEntry);

    return STATUS_SUCCESS;
}

//
// CustomGetAppIdFromFileName()
//
// The API FwpmGetAppIdFromFileName() is not exposed in kernel mode, but we
// don't need it. All it does is look up the device path which we already have.
//
// However, for some reason the string also has to be null-terminated.
//
NTSTATUS
CustomGetAppIdFromFileName(const LOWER_UNICODE_STRING* ImageName, FWP_BYTE_BLOB** AppId) {
    auto offsetStringBuffer = util::RoundToMultiple(sizeof(FWP_BYTE_BLOB), TYPE_ALIGNMENT(WCHAR));

    UINT32 copiedStringLength = ImageName->Length + sizeof(WCHAR);

    auto allocationSize = offsetStringBuffer + copiedStringLength;

    auto blob = (FWP_BYTE_BLOB*)ExAllocatePoolUninitialized(PagedPool, allocationSize, ST_POOL_TAG);

    if (blob == NULL) {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    auto stringBuffer = ((UINT8*)blob) + offsetStringBuffer;

    RtlCopyMemory(stringBuffer, ImageName->Buffer, ImageName->Length);

    stringBuffer[copiedStringLength - 2] = 0;
    stringBuffer[copiedStringLength - 1] = 0;

    blob->size = copiedStringLength;
    blob->data = stringBuffer;

    *AppId = blob;

    return STATUS_SUCCESS;
}

//
// FindBlockConnectionsEntry()
//
// Returns pointer to matching entry or NULL.
//
BLOCK_CONNECTIONS_ENTRY* FindBlockConnectionsEntry(LIST_ENTRY* List, const LOWER_UNICODE_STRING* ImageName) {
    for (auto entry = List->Flink; entry != List; entry = entry->Flink) {
        auto candidate = (BLOCK_CONNECTIONS_ENTRY*)entry;

        if (util::Equal(ImageName, &candidate->ImageName)) {
            return candidate;
        }
    }

    return NULL;
}

NTSTATUS
AddTunnelBlockFiltersTx(
    HANDLE WfpSession,
    const LOWER_UNICODE_STRING* ImageName,
    const IN_ADDR* TunnelIpv4,
    const IN6_ADDR* TunnelIpv6,
    UINT64* OutboundFilterIdV4,
    UINT64* InboundFilterIdV4,
    UINT64* OutboundFilterIdV6,
    UINT64* InboundFilterIdV6) {
    NT_ASSERT(
        OutboundFilterIdV4 != NULL && InboundFilterIdV4 != NULL && OutboundFilterIdV6 != NULL &&
        InboundFilterIdV6 != NULL);

    //
    // Format APP_ID payload that will be used with all filters.
    //

    FWP_BYTE_BLOB* appIdPayload;

    auto status = CustomGetAppIdFromFileName(ImageName, &appIdPayload);

    if (!NT_SUCCESS(status)) {
        return status;
    }

    if (TunnelIpv4 == NULL) {
        *OutboundFilterIdV4 = 0;
        *InboundFilterIdV4 = 0;
    } else {
        //
        // Register outbound IPv4 filter.
        //

        FWPM_FILTER0 filter = { 0 };

        auto const FilterNameOutboundIpv4 = L"Nym VPN Split Tunnel In-Tunnel Blocking Filter (Outbound IPv4)";
        auto const FilterDescription = L"Blocks existing connections in the tunnel";

        filter.displayData.name = const_cast<wchar_t*>(FilterNameOutboundIpv4);
        filter.displayData.description = const_cast<wchar_t*>(FilterDescription);
        filter.flags = FWPM_FILTER_FLAG_CLEAR_ACTION_RIGHT | FWPM_FILTER_FLAG_HAS_PROVIDER_CONTEXT;
        filter.providerKey = const_cast<GUID*>(&ST_FW_PROVIDER_KEY);
        filter.layerKey = FWPM_LAYER_ALE_AUTH_CONNECT_V4;
        filter.subLayerKey = ST_FW_WINFW_BASELINE_SUBLAYER_KEY;
        filter.weight.type = FWP_UINT64;
        filter.weight.uint64 = const_cast<UINT64*>(&ST_MAX_FILTER_WEIGHT);
        filter.action.type = FWP_ACTION_CALLOUT_UNKNOWN;
        filter.action.calloutKey = ST_FW_CALLOUT_BLOCK_SPLIT_APPS_IPV4_CONN_KEY;
        filter.providerContextKey = ST_FW_PROVIDER_CONTEXT_KEY;

        //
        // Conditions are:
        //
        // APP_ID == ImageName
        // LOCAL_ADDRESS == TunnelIp
        //

        FWPM_FILTER_CONDITION0 cond[2];

        cond[0].fieldKey = FWPM_CONDITION_ALE_APP_ID;
        cond[0].matchType = FWP_MATCH_EQUAL;
        cond[0].conditionValue.type = FWP_BYTE_BLOB_TYPE;
        cond[0].conditionValue.byteBlob = appIdPayload;

        cond[1].fieldKey = FWPM_CONDITION_IP_LOCAL_ADDRESS;
        cond[1].matchType = FWP_MATCH_EQUAL;
        cond[1].conditionValue.type = FWP_UINT32;
        cond[1].conditionValue.uint32 = RtlUlongByteSwap(TunnelIpv4->s_addr);

        filter.filterCondition = cond;
        filter.numFilterConditions = ARRAYSIZE(cond);

        status = FwpmFilterAdd0(WfpSession, &filter, NULL, OutboundFilterIdV4);

        if (!NT_SUCCESS(status)) {
            goto Cleanup;
        }

        //
        // Register inbound IPv4 filter.
        //

        auto const FilterNameInboundIpv4 = L"Nym VPN Split Tunnel In-Tunnel Blocking Filter (Inbound IPv4)";

        RtlZeroMemory(&filter.filterKey, sizeof(filter.filterKey));
        filter.displayData.name = const_cast<wchar_t*>(FilterNameInboundIpv4);
        filter.layerKey = FWPM_LAYER_ALE_AUTH_RECV_ACCEPT_V4;
        filter.action.calloutKey = ST_FW_CALLOUT_BLOCK_SPLIT_APPS_IPV4_RECV_KEY;

        status = FwpmFilterAdd0(WfpSession, &filter, NULL, InboundFilterIdV4);

        if (!NT_SUCCESS(status)) {
            goto Cleanup;
        }
    }

    if (TunnelIpv6 == NULL) {
        *OutboundFilterIdV6 = 0;
        *InboundFilterIdV6 = 0;
    } else {
        //
        // Register outbound IPv6 filter.
        //

        FWPM_FILTER0 filter = { 0 };

        auto const FilterNameOutboundIpv6 = L"Nym VPN Split Tunnel In-Tunnel Blocking Filter (Outbound IPv6)";
        auto const FilterDescription = L"Blocks existing connections in the tunnel";

        filter.displayData.name = const_cast<wchar_t*>(FilterNameOutboundIpv6);
        filter.displayData.description = const_cast<wchar_t*>(FilterDescription);
        filter.flags = FWPM_FILTER_FLAG_CLEAR_ACTION_RIGHT | FWPM_FILTER_FLAG_HAS_PROVIDER_CONTEXT;
        filter.providerKey = const_cast<GUID*>(&ST_FW_PROVIDER_KEY);
        filter.layerKey = FWPM_LAYER_ALE_AUTH_CONNECT_V6;
        filter.subLayerKey = ST_FW_WINFW_BASELINE_SUBLAYER_KEY;
        filter.weight.type = FWP_UINT64;
        filter.weight.uint64 = const_cast<UINT64*>(&ST_MAX_FILTER_WEIGHT);
        filter.action.type = FWP_ACTION_CALLOUT_UNKNOWN;
        filter.action.calloutKey = ST_FW_CALLOUT_BLOCK_SPLIT_APPS_IPV6_CONN_KEY;
        filter.providerContextKey = ST_FW_PROVIDER_CONTEXT_KEY;

        //
        // Conditions are:
        //
        // APP_ID == ImageName
        // LOCAL_ADDRESS == TunnelIp
        //

        FWPM_FILTER_CONDITION0 cond[2];

        cond[0].fieldKey = FWPM_CONDITION_ALE_APP_ID;
        cond[0].matchType = FWP_MATCH_EQUAL;
        cond[0].conditionValue.type = FWP_BYTE_BLOB_TYPE;
        cond[0].conditionValue.byteBlob = appIdPayload;

        cond[1].fieldKey = FWPM_CONDITION_IP_LOCAL_ADDRESS;
        cond[1].matchType = FWP_MATCH_EQUAL;
        cond[1].conditionValue.type = FWP_BYTE_ARRAY16_TYPE;
        cond[1].conditionValue.byteArray16 = (FWP_BYTE_ARRAY16*)TunnelIpv6->u.Byte;

        filter.filterCondition = cond;
        filter.numFilterConditions = ARRAYSIZE(cond);

        status = FwpmFilterAdd0(WfpSession, &filter, NULL, OutboundFilterIdV6);

        if (!NT_SUCCESS(status)) {
            goto Cleanup;
        }

        //
        // Register inbound IPv6 filter.
        //

        auto const FilterNameInboundIpv6 = L"Nym VPN Split Tunnel In-Tunnel Blocking Filter (Inbound IPv6)";

        RtlZeroMemory(&filter.filterKey, sizeof(filter.filterKey));
        filter.displayData.name = const_cast<wchar_t*>(FilterNameInboundIpv6);
        filter.layerKey = FWPM_LAYER_ALE_AUTH_RECV_ACCEPT_V6;
        filter.action.calloutKey = ST_FW_CALLOUT_BLOCK_SPLIT_APPS_IPV6_RECV_KEY;

        status = FwpmFilterAdd0(WfpSession, &filter, NULL, InboundFilterIdV6);

        if (!NT_SUCCESS(status)) {
            goto Cleanup;
        }
    }

    status = STATUS_SUCCESS;

Cleanup:

    ExFreePoolWithTag(appIdPayload, ST_POOL_TAG);

    return status;
}

typedef NTSTATUS (*AddBlockFiltersFunc)(
    HANDLE WfpSession,
    const LOWER_UNICODE_STRING* ImageName,
    const IN_ADDR* TunnelIpv4,
    const IN6_ADDR* TunnelIpv6,
    UINT64* OutboundFilterIdV4,
    UINT64* InboundFilterIdV4,
    UINT64* OutboundFilterIdV6,
    UINT64* InboundFilterIdV6);

NTSTATUS
AddBlockFiltersCreateEntryTx(
    HANDLE WfpSession,
    const LOWER_UNICODE_STRING* ImageName,
    const IN_ADDR* TunnelIpv4,
    const IN6_ADDR* TunnelIpv6,
    AddBlockFiltersFunc Blocker,
    BLOCK_CONNECTIONS_ENTRY** Entry) {
    auto offsetStringBuffer = util::RoundToMultiple(sizeof(BLOCK_CONNECTIONS_ENTRY), TYPE_ALIGNMENT(WCHAR));

    auto allocationSize = offsetStringBuffer + ImageName->Length;

    WCHAR* stringBuffer = NULL;

    auto entry = (BLOCK_CONNECTIONS_ENTRY*)ExAllocatePoolUninitialized(PagedPool, allocationSize, ST_POOL_TAG);

    if (entry == NULL) {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    RtlZeroMemory(entry, allocationSize);

    auto status = Blocker(
        WfpSession,
        ImageName,
        TunnelIpv4,
        TunnelIpv6,
        &entry->OutboundFilterIdV4,
        &entry->InboundFilterIdV4,
        &entry->OutboundFilterIdV6,
        &entry->InboundFilterIdV6);

    if (!NT_SUCCESS(status)) {
        DbgPrint("Failed to add block filters: 0x%X\n", status);

        ExFreePoolWithTag(entry, ST_POOL_TAG);

        return status;
    }

    stringBuffer = (WCHAR*)(((UINT8*)entry) + offsetStringBuffer);

    InitializeListHead(&entry->ListEntry);

    entry->RefCount = 1;

    entry->ImageName.Length = ImageName->Length;
    entry->ImageName.MaximumLength = ImageName->Length;
    entry->ImageName.Buffer = stringBuffer;

    RtlCopyMemory(stringBuffer, ImageName->Buffer, ImageName->Length);

    *Entry = entry;

    return STATUS_SUCCESS;
}

NTSTATUS
RemoveBlockFiltersTx(
    HANDLE WfpSession,
    UINT64 OutboundFilterIdV4,
    UINT64 InboundFilterIdV4,
    UINT64 OutboundFilterIdV6,
    UINT64 InboundFilterIdV6) {
    //
    // Filters were installed in pairs.
    //

    NT_ASSERT(OutboundFilterIdV4 != 0 || OutboundFilterIdV6 != 0);

    auto status = STATUS_SUCCESS;

    if (OutboundFilterIdV4 != 0) {
        status = FwpmFilterDeleteById0(WfpSession, OutboundFilterIdV4);

        if (!NT_SUCCESS(status)) {
            return status;
        }

        NT_ASSERT(InboundFilterIdV4 != 0);

        status = FwpmFilterDeleteById0(WfpSession, InboundFilterIdV4);

        if (!NT_SUCCESS(status)) {
            return status;
        }
    }

    if (OutboundFilterIdV6 != 0) {
        status = FwpmFilterDeleteById0(WfpSession, OutboundFilterIdV6);

        if (!NT_SUCCESS(status)) {
            return status;
        }

        NT_ASSERT(InboundFilterIdV6 != 0);

        status = FwpmFilterDeleteById0(WfpSession, InboundFilterIdV6);

        if (!NT_SUCCESS(status)) {
            return status;
        }
    }

    return STATUS_SUCCESS;
}

NTSTATUS
RemoveBlockFiltersAndEntryTx(HANDLE WfpSession, LIST_ENTRY* TransactionEvents, BLOCK_CONNECTIONS_ENTRY* Entry) {
    auto status = RemoveBlockFiltersTx(
        WfpSession,
        Entry->OutboundFilterIdV4,
        Entry->InboundFilterIdV4,
        Entry->OutboundFilterIdV6,
        Entry->InboundFilterIdV6);

    if (!NT_SUCCESS(status)) {
        DbgPrint("Could not remove block filters: 0x%X\n", status);

        return status;
    }

    //
    // Record in transaction history before unlinking, because the former is a
    // fallible operation.
    //

    status = TransactionRemovedEntry(TransactionEvents, Entry, Entry->ListEntry.Blink);

    if (!NT_SUCCESS(status)) {
        DbgPrint("Could not update local transaction: 0x%X\n", status);

        return status;
    }

    RemoveEntryList(&Entry->ListEntry);

    return STATUS_SUCCESS;
}

void FreeList(LIST_ENTRY* List) {
    LIST_ENTRY* entry;

    while ((entry = RemoveHeadList(List)) != List) {
        ExFreePoolWithTag(entry, ST_POOL_TAG);
    }
}

} // anonymous namespace

NTSTATUS
Initialize(HANDLE WfpSession, void** Context) {
    auto context =
        (APP_FILTERS_CONTEXT*)ExAllocatePoolUninitialized(PagedPool, sizeof(APP_FILTERS_CONTEXT), ST_POOL_TAG);

    if (context == NULL) {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    context->WfpSession = WfpSession;

    InitializeListHead(&context->BlockedTunnelConnections);

    InitializeListHead(&context->TransactionEvents);

    *Context = context;

    return STATUS_SUCCESS;
}

void TearDown(void** Context) {
    auto context = (APP_FILTERS_CONTEXT*)*Context;

    //
    // This is a best effort venture so just keep going.
    //
    // Remove all app specific filters.
    //

    for (auto rawEntry = context->BlockedTunnelConnections.Flink; rawEntry != &context->BlockedTunnelConnections;
         /* no post-condition */) {
        auto entry = (BLOCK_CONNECTIONS_ENTRY*)rawEntry;

        RemoveBlockFiltersTx(
            context->WfpSession,
            entry->OutboundFilterIdV4,
            entry->InboundFilterIdV4,
            entry->OutboundFilterIdV6,
            entry->InboundFilterIdV6);

        auto next = rawEntry->Flink;

        ExFreePoolWithTag(rawEntry, ST_POOL_TAG);

        rawEntry = next;
    }

    InitializeListHead(&context->BlockedTunnelConnections);

    //
    // This works because a commit discards all transaction events.
    // (Also, there shouldn't be any events at this time.)
    //

    if (!IsListEmpty(&context->TransactionEvents)) {
        DbgPrint("ERROR: Active transaction while tearing down appfilters module\n");
    }

    TransactionCommit(*Context);

    //
    // Release context.
    //

    ExFreePoolWithTag(context, ST_POOL_TAG);

    *Context = NULL;
}

NTSTATUS
TransactionBegin(void* Context) {
    auto context = (APP_FILTERS_CONTEXT*)Context;

    if (IsListEmpty(&context->TransactionEvents)) {
        return STATUS_SUCCESS;
    }

    return STATUS_TRANSACTION_REQUEST_NOT_VALID;
}

void TransactionCommit(void* Context) {
    //
    // All changes are already applied, discard transaction events.
    //
    // Each event has to be released, and some of them point to
    // a target entry which must also be released.
    //

    auto context = (APP_FILTERS_CONTEXT*)Context;

    auto list = &context->TransactionEvents;
    LIST_ENTRY* rawEvent;

    while ((rawEvent = RemoveHeadList(list)) != list) {
        switch (((TRANSACTION_EVENT*)rawEvent)->EventType) {
        case TRANSACTION_EVENT_TYPE::ADD_ENTRY: {
            auto addEvent = (TRANSACTION_EVENT_ADD_ENTRY*)rawEvent;

            ExFreePoolWithTag(addEvent->Target, ST_POOL_TAG);

            break;
        }
        case TRANSACTION_EVENT_TYPE::SWAP_LISTS: {
            auto swapEvent = (TRANSACTION_EVENT_SWAP_LISTS*)rawEvent;

            FreeList(&swapEvent->BlockedTunnelConnections);

            break;
        }
        }

        ExFreePoolWithTag(rawEvent, ST_POOL_TAG);
    }
}

void TransactionAbort(void* Context) {
    //
    // Step back through event records and undo all changes.
    //

    auto context = (APP_FILTERS_CONTEXT*)Context;

    auto list = &context->TransactionEvents;
    LIST_ENTRY* rawEvent;

    while ((rawEvent = RemoveHeadList(list)) != list) {
        auto evt = (TRANSACTION_EVENT*)rawEvent;

        switch (evt->EventType) {
        case TRANSACTION_EVENT_TYPE::INCREMENT_REF_COUNT: {
            ++evt->Target->RefCount;

            break;
        }
        case TRANSACTION_EVENT_TYPE::DECREMENT_REF_COUNT: {
            --evt->Target->RefCount;

            break;
        }
        case TRANSACTION_EVENT_TYPE::ADD_ENTRY: {
            auto addEvent = (TRANSACTION_EVENT_ADD_ENTRY*)rawEvent;

            InsertHeadList(addEvent->MockHead, &addEvent->Target->ListEntry);

            break;
        }
        case TRANSACTION_EVENT_TYPE::REMOVE_ENTRY: {
            RemoveEntryList(&evt->Target->ListEntry);

            ExFreePoolWithTag(evt->Target, ST_POOL_TAG);

            break;
        }
        case TRANSACTION_EVENT_TYPE::SWAP_LISTS: {
            auto liveList = &context->BlockedTunnelConnections;

            FreeList(liveList);

            auto swapEvent = (TRANSACTION_EVENT_SWAP_LISTS*)rawEvent;

            util::ReparentList(liveList, &swapEvent->BlockedTunnelConnections);

            break;
        }
        };

        ExFreePoolWithTag(rawEvent, ST_POOL_TAG);
    }
}

//
// RegisterFilterBlockAppTunnelTrafficTx2()
//
// Register filters that block tunnel traffic for a specific app.
//
// This is primarily done to ensure an application's existing connections are
// blocked when the app starts being split.
//
// When filters are added, a re-auth occurs, and matching existing connections
// are presented to the linked callout, to approve or block.
//
NTSTATUS
RegisterFilterBlockAppTunnelTrafficTx2(
    void* Context, const LOWER_UNICODE_STRING* ImageName, const IN_ADDR* TunnelIpv4, const IN6_ADDR* TunnelIpv6) {
    if (TunnelIpv4 == NULL && TunnelIpv6 == NULL) {
        return STATUS_INVALID_PARAMETER;
    }

    auto context = (APP_FILTERS_CONTEXT*)Context;

    auto existingEntry = FindBlockConnectionsEntry(&context->BlockedTunnelConnections, ImageName);

    if (existingEntry != NULL) {
        auto status = TransactionIncrementedRefCount(&context->TransactionEvents, existingEntry);

        if (!NT_SUCCESS(status)) {
            DbgPrint("Could not update local transaction: 0x%X\n", status);

            return status;
        }

        ++existingEntry->RefCount;

        return STATUS_SUCCESS;
    }

    BLOCK_CONNECTIONS_ENTRY* entry;

    auto status = AddBlockFiltersCreateEntryTx(
        context->WfpSession, ImageName, TunnelIpv4, TunnelIpv6, AddTunnelBlockFiltersTx, &entry);

    if (!NT_SUCCESS(status)) {
        return status;
    }

    status = TransactionAddedEntry(&context->TransactionEvents, entry);

    if (!NT_SUCCESS(status)) {
        DbgPrint("Could not update local transaction: 0x%X\n", status);

        ExFreePoolWithTag(entry, ST_POOL_TAG);

        return status;
    }

    InsertTailList(&context->BlockedTunnelConnections, &entry->ListEntry);

    DbgPrint("Added tunnel block filters for %wZ\n", (const UNICODE_STRING*)ImageName);

    return STATUS_SUCCESS;
}

NTSTATUS
RemoveFilterBlockAppTunnelTrafficTx2(void* Context, const LOWER_UNICODE_STRING* ImageName) {
    auto context = (APP_FILTERS_CONTEXT*)Context;

    auto entry = FindBlockConnectionsEntry(&context->BlockedTunnelConnections, ImageName);

    if (entry == NULL) {
        return STATUS_INVALID_PARAMETER;
    }

    if (entry->RefCount > 1) {
        auto status = TransactionDecrementedRefCount(&context->TransactionEvents, entry);

        if (!NT_SUCCESS(status)) {
            DbgPrint("Could not update local transaction: 0x%X\n", status);

            return status;
        }

        --entry->RefCount;

        //
        // TODO: Indicate to layer above that it might want to force an ALE
        // reauthorization in WFP.
        //
        // https://docs.microsoft.com/en-us/windows/win32/fwp/ale-re-authorization
        //
        // Forcing a reauthorization is only necessary in very specific cases.
        // Usually the transaction will include the addition/removal of at least one
        // filter, and this triggers a reauthorization.
        //
        // Also, the issue only comes into play if a process stops being split but
        // keeps running.
        //
        // Rationale:
        //
        // There could be existing connections which have been blocked for some
        // duration of time and now need to be reauthorized in WFP so they are no
        // longer blocked.
        //
        // Similarly, there could be non-tunnel connections that were previously
        // approved and should now be reauthorized so they can be blocked.
        //

        return STATUS_SUCCESS;
    }

    auto status = RemoveBlockFiltersAndEntryTx(context->WfpSession, &context->TransactionEvents, entry);

    if (NT_SUCCESS(status)) {
        DbgPrint("Removed tunnel block filters for %wZ\n", (const UNICODE_STRING*)ImageName);
    }

    return status;
}

NTSTATUS
ResetTx2(void* Context) {
    auto context = (APP_FILTERS_CONTEXT*)Context;

    if (IsListEmpty(&context->BlockedTunnelConnections)) {
        return STATUS_SUCCESS;
    }

    for (auto rawEntry = context->BlockedTunnelConnections.Flink; rawEntry != &context->BlockedTunnelConnections;
         rawEntry = rawEntry->Flink) {
        auto entry = (BLOCK_CONNECTIONS_ENTRY*)rawEntry;

        auto status = RemoveBlockFiltersTx(
            context->WfpSession,
            entry->OutboundFilterIdV4,
            entry->InboundFilterIdV4,
            entry->OutboundFilterIdV6,
            entry->InboundFilterIdV6);

        if (!NT_SUCCESS(status)) {
            return status;
        }
    }

    //
    // Create transaction event and pass ownership of list to it.
    //
    auto status = TransactionSwappedLists(&context->TransactionEvents, &context->BlockedTunnelConnections);

    if (!NT_SUCCESS(status)) {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    //
    // Clear list to reflect new state.
    //
    InitializeListHead(&context->BlockedTunnelConnections);

    return STATUS_SUCCESS;
}

NTSTATUS
UpdateFiltersTx2(void* Context, const IN_ADDR* TunnelIpv4, const IN6_ADDR* TunnelIpv6) {
    auto context = (APP_FILTERS_CONTEXT*)Context;

    if (IsListEmpty(&context->BlockedTunnelConnections)) {
        return STATUS_SUCCESS;
    }

    LIST_ENTRY newList;

    InitializeListHead(&newList);

    for (auto rawEntry = context->BlockedTunnelConnections.Flink; rawEntry != &context->BlockedTunnelConnections;
         rawEntry = rawEntry->Flink) {
        auto entry = (BLOCK_CONNECTIONS_ENTRY*)rawEntry;

        auto status = RemoveBlockFiltersTx(
            context->WfpSession,
            entry->OutboundFilterIdV4,
            entry->InboundFilterIdV4,
            entry->OutboundFilterIdV6,
            entry->InboundFilterIdV6);

        if (!NT_SUCCESS(status)) {
            FreeList(&newList);

            return status;
        }

        BLOCK_CONNECTIONS_ENTRY* newEntry;

        status = AddBlockFiltersCreateEntryTx(
            context->WfpSession, &entry->ImageName, TunnelIpv4, TunnelIpv6, AddTunnelBlockFiltersTx, &newEntry);

        if (!NT_SUCCESS(status)) {
            FreeList(&newList);

            return status;
        }

        newEntry->RefCount = entry->RefCount;

        InsertTailList(&newList, &newEntry->ListEntry);
    }

    //
    // context->BlockedTunnelConnections is now completely obsolete.
    // newList has all the updated entries.
    //

    auto status = TransactionSwappedLists(&context->TransactionEvents, &context->BlockedTunnelConnections);

    if (!NT_SUCCESS(status)) {
        FreeList(&newList);

        return status;
    }

    //
    // Ownership of the list formerly rooted at context->BlockedTunnelConnections
    // has been moved to the recently queued transaction event.
    //
    // Perform actual state update.
    //

    util::ReparentList(&context->BlockedTunnelConnections, &newList);

    return STATUS_SUCCESS;
}

} // namespace firewall::appfilters
