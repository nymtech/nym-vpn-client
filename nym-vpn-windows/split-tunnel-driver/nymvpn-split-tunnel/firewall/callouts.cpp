// Copyright 2016-2026 Mullvad VPN AB. All Rights Reserved.
// Copyright 2026 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

#include "callouts.h"
#include "../util.h"
#include "classify.h"
#include "context.h"
#include "firewall.h"
#include "identifiers.h"
#include "logging.h"
#include "pending.h"
#include "wfp.h"

#include "../trace.h"
#include "callouts.tmh"

#define RETURN_IF_UNSUCCESSFUL(status)                                                                                 \
    if (!NT_SUCCESS(status)) {                                                                                         \
        return status;                                                                                                 \
    }

namespace firewall {

namespace {

//
// NotifyFilterAttach()
//
// Receive notifications about filters attaching/detaching the callout.
//
NTSTATUS
NotifyFilterAttach(FWPS_CALLOUT_NOTIFY_TYPE notifyType, const GUID* filterKey, FWPS_FILTER1* filter) {
    UNREFERENCED_PARAMETER(notifyType);
    UNREFERENCED_PARAMETER(filterKey);
    UNREFERENCED_PARAMETER(filter);

    return STATUS_SUCCESS;
}

NTSTATUS
RegisterCalloutTx(
    PDEVICE_OBJECT DeviceObject,
    HANDLE WfpSession,
    FWPS_CALLOUT_CLASSIFY_FN1 Callout,
    const GUID* CalloutKey,
    const GUID* LayerKey,
    wchar_t const* CalloutName,
    wchar_t const* CalloutDescription) {
    //
    // Logically, this is the wrong order, but it results in cleaner code.
    // You're encouraged to first register the callout and then add it.
    //
    // However, what's currently here is fully supported:
    //
    // `By default filters that reference callouts that have been added
    // but have not yet registered with the filter engine are treated as Block
    // filters.`
    //

    FWPM_CALLOUT0 callout;

    RtlZeroMemory(&callout, sizeof(callout));

    callout.calloutKey = *CalloutKey;
    callout.displayData.name = const_cast<wchar_t*>(CalloutName);
    callout.displayData.description = const_cast<wchar_t*>(CalloutDescription);
    callout.flags = FWPM_CALLOUT_FLAG_USES_PROVIDER_CONTEXT;
    callout.providerKey = const_cast<GUID*>(&ST_FW_PROVIDER_KEY);
    callout.applicableLayer = *LayerKey;

    auto status = FwpmCalloutAdd0(WfpSession, &callout, NULL, NULL);

    if (!NT_SUCCESS(status) && status != STATUS_FWP_ALREADY_EXISTS) {
        return status;
    }

    FWPS_CALLOUT1 aCallout = { 0 };

    aCallout.calloutKey = *CalloutKey;
    aCallout.classifyFn = Callout;
    aCallout.notifyFn = NotifyFilterAttach;
    aCallout.flowDeleteFn = NULL;

    status = FwpsCalloutRegister1(DeviceObject, &aCallout, NULL);

    if (status == STATUS_FWP_ALREADY_EXISTS) {
        return STATUS_SUCCESS;
    }

    return status;
}

//
// UnregisterCallout()
//
// This is a thin wrapper around FwpsCalloutUnregisterByKey0().
// The reason is to clarify and simplify usage.
//
NTSTATUS
UnregisterCallout(const GUID* CalloutKey) {
    auto const status = FwpsCalloutUnregisterByKey0(CalloutKey);

    if (NT_SUCCESS(status)) {
        return status;
    }

    if (status == STATUS_FWP_CALLOUT_NOT_FOUND) {
        return STATUS_SUCCESS;
    }

    //
    // The current implementation doesn't process flows or use flow contexts.
    // So this status code won't be returned.
    //
    NT_ASSERT(status != STATUS_DEVICE_BUSY);

    //
    // The current implementation manages registration and unregistration
    // on the primary thread. So this status code won't be returned.
    //
    NT_ASSERT(status != STATUS_FWP_IN_USE);

    return status;
}

//
// RewriteBind()
//
// Implements redirection for non-TCP socket binds.
//
// If a bind is attempted (or implied) with target inaddr_any or the tunnel
// interface, we rewrite the bind to move it to the internet interface.
//
// This has the unfortunate effect that client sockets which are not explicitly
// bound to localhost are prevented from connecting to localhost.
//
void RewriteBind(
    CONTEXT* Context,
    const FWPS_INCOMING_VALUES0* FixedValues,
    const FWPS_INCOMING_METADATA_VALUES0* MetaValues,
    UINT64 FilterId,
    void const* ClassifyContext,
    FWPS_CLASSIFY_OUT0* ClassifyOut) {
    UNREFERENCED_PARAMETER(MetaValues);

    UINT64 classifyHandle = 0;

    auto status = FwpsAcquireClassifyHandle0(const_cast<void*>(ClassifyContext), 0, &classifyHandle);

    if (!NT_SUCCESS(status)) {
        DbgPrint("FwpsAcquireClassifyHandle0() failed 0x%X\n", status);

        return;
    }

    FWPS_BIND_REQUEST0* bindRequest = NULL;

    status = FwpsAcquireWritableLayerDataPointer0(classifyHandle, FilterId, 0, (PVOID*)&bindRequest, ClassifyOut);

    if (!NT_SUCCESS(status)) {
        DbgPrint("FwpsAcquireWritableLayerDataPointer0() failed 0x%X\n", status);
        FwpsReleaseClassifyHandle0(classifyHandle);

        return;
    }

    ClassificationReset(ClassifyOut);

    //
    // There's a list with redirection history.
    //
    // This only ever comes into play if several callouts are fighting to redirect
    // the bind.
    //
    // To prevent recursion, we need to check if we're on the list, and abort if
    // so.
    //

    bool alreadyRedirectedByUs = false;

    for (auto history = bindRequest->previousVersion; history != NULL; history = history->previousVersion) {
        if (history->modifierFilterId == FilterId) {
            DbgPrint("Aborting bind processing because already redirected by us\n");

            alreadyRedirectedByUs = true;
            break;
        }
    }

    if (!alreadyRedirectedByUs) {
        //
        // Rewrite bind as applicable.
        //

        bool const ipv4 = FixedValues->layerId == FWPS_LAYER_ALE_BIND_REDIRECT_V4;

        WdfSpinLockAcquire(Context->IpAddresses.Lock);

        if (ipv4) {
            auto bindTarget = (SOCKADDR_IN*)&(bindRequest->localAddressAndPort);

            if (IN4_IS_ADDR_UNSPECIFIED(&(bindTarget->sin_addr)) ||
                IN4_ADDR_EQUAL(&(bindTarget->sin_addr), &(Context->IpAddresses.Addresses.TunnelIpv4))) {
                auto const newTarget = &Context->IpAddresses.Addresses.InternetIpv4;

                LogBindRedirect(HANDLE(MetaValues->processId), bindTarget, newTarget);

                bindTarget->sin_addr = *newTarget;

                ClassificationApplySoftPermit(ClassifyOut);
            }
        } else {
            auto bindTarget = (SOCKADDR_IN6*)&(bindRequest->localAddressAndPort);

            static const IN6_ADDR IN6_ADDR_ANY = { 0 };

            if (IN6_ADDR_EQUAL(&(bindTarget->sin6_addr), &IN6_ADDR_ANY) ||
                IN6_ADDR_EQUAL(&(bindTarget->sin6_addr), &(Context->IpAddresses.Addresses.TunnelIpv6))) {
                auto const newTarget = &Context->IpAddresses.Addresses.InternetIpv6;

                LogBindRedirect(HANDLE(MetaValues->processId), bindTarget, newTarget);

                bindTarget->sin6_addr = *newTarget;

                ClassificationApplySoftPermit(ClassifyOut);
            }
        }

        WdfSpinLockRelease(Context->IpAddresses.Lock);
    }

    //
    // Call the "apply" function even in instances where we've made no changes
    // to the data, because it was deemed not necessary, or aborting for some
    // other reason.
    //
    // This is the correct logic according to documentation.
    //

    FwpsApplyModifiedLayerData0(classifyHandle, bindRequest, 0);

    FwpsReleaseClassifyHandle0(classifyHandle);
}

//
// PendClassification()
//
// This function is used when, for an incoming request, we don't know what the
// correct action is. I.e. when the process making the request hasn't been
// categorized yet.
//
void PendClassification(
    pending::CONTEXT* Context,
    HANDLE ProcessId,
    UINT64 FilterId,
    UINT16 LayerId,
    void const* ClassifyContext,
    FWPS_CLASSIFY_OUT0* ClassifyOut) {
    auto status =
        pending::PendRequest(Context, ProcessId, const_cast<void*>(ClassifyContext), FilterId, LayerId, ClassifyOut);

    if (NT_SUCCESS(status)) {
        return;
    }

    pending::FailRequest(ProcessId, const_cast<void*>(ClassifyContext), FilterId, LayerId, ClassifyOut);
}

//
// CalloutClassifyBind()
//
// ===
//
// NOTE: This function is always called at PASSIVE_LEVEL.
//
// Callouts are generally activated at <= DISPATCH_LEVEL, but the bind redirect
// layers are special-cased and guarantee PASSIVE_LEVEL.
//
// https://community.osr.com/discussion/292855/irql-for-wfp-callouts-at-fwpm-layer-ale-bind-redirect-vxxx
//
// ===
//
// Entry point for splitting non-TCP socket binds.
//
// FWPS_LAYER_ALE_BIND_REDIRECT_V4
// FWPS_LAYER_ALE_BIND_REDIRECT_V6
//
void CalloutClassifyBind(
    const FWPS_INCOMING_VALUES0* FixedValues,
    const FWPS_INCOMING_METADATA_VALUES0* MetaValues,
    void* LayerData,
    void const* ClassifyContext,
    const FWPS_FILTER1* Filter,
    UINT64 FlowContext,
    FWPS_CLASSIFY_OUT0* ClassifyOut) {
    UNREFERENCED_PARAMETER(LayerData);
    UNREFERENCED_PARAMETER(FlowContext);

    NT_ASSERT(
        (FixedValues->layerId == FWPS_LAYER_ALE_BIND_REDIRECT_V4 &&
         FixedValues->incomingValue[FWPS_FIELD_ALE_BIND_REDIRECT_V4_IP_PROTOCOL].value.uint8 != IPPROTO_TCP) ||
        (FixedValues->layerId == FWPS_LAYER_ALE_BIND_REDIRECT_V6 &&
         FixedValues->incomingValue[FWPS_FIELD_ALE_BIND_REDIRECT_V6_IP_PROTOCOL].value.uint8 != IPPROTO_TCP));

    NT_ASSERT(
        Filter->providerContext != NULL && Filter->providerContext->type == FWPM_GENERAL_CONTEXT &&
        Filter->providerContext->dataBuffer->size == sizeof(CONTEXT*));

    auto context = *(CONTEXT**)Filter->providerContext->dataBuffer->data;

    if ((ClassifyOut->rights & FWPS_RIGHT_ACTION_WRITE) == 0) {
        DbgPrint("Aborting bind processing because hard permit/block already applied\n");

        return;
    }

    ClassificationReset(ClassifyOut);

    if (!FWPS_IS_METADATA_FIELD_PRESENT(MetaValues, FWPS_METADATA_FIELD_PROCESS_ID)) {
        DbgPrint("Failed to classify bind because PID was not provided\n");

        return;
    }

    const CALLBACKS& callbacks = context->Callbacks;

    auto const verdict = callbacks.QueryProcess(HANDLE(MetaValues->processId), callbacks.Context);

    switch (verdict) {
    case PROCESS_SPLIT_VERDICT::DO_SPLIT: {
        RewriteBind(context, FixedValues, MetaValues, Filter->filterId, ClassifyContext, ClassifyOut);

        break;
    }
    case PROCESS_SPLIT_VERDICT::BYPASS: {
        //
        // The process binds to whichever interface it chooses.
        // No rewriting; just let the bind proceed.
        //
        ClassificationApplySoftPermit(ClassifyOut);

        break;
    }
    case PROCESS_SPLIT_VERDICT::UNKNOWN: {
        PendClassification(
            context->PendedClassifications,
            HANDLE(MetaValues->processId),
            Filter->filterId,
            FixedValues->layerId,
            ClassifyContext,
            ClassifyOut);

        break;
    }
    };
}

bool LocalAddress(const IN_ADDR* addr) {
    return IN4_IS_ADDR_LOOPBACK(addr)         // 127/8
           || IN4_IS_ADDR_LINKLOCAL(addr)     // 169.254/16
           || IN4_IS_ADDR_RFC1918(addr)       // 10/8, 172.16/12, 192.168/16
           || IN4_IS_ADDR_MC_LINKLOCAL(addr)  // 224.0.0/24
           || IN4_IS_ADDR_MC_ADMINLOCAL(addr) // 239.255/16
           || IN4_IS_ADDR_MC_SITELOCAL(addr)  // 239/8
           || IN4_IS_ADDR_BROADCAST(addr)     // 255.255.255.255
        ;
}

bool IN6_IS_ADDR_ULA(const IN6_ADDR* a) {
    return (a->s6_bytes[0] == 0xfd);
}

bool IN6_IS_ADDR_MC_NON_GLOBAL(const IN6_ADDR* a) {
    return IN6_IS_ADDR_MULTICAST(a) && !IN6_IS_ADDR_MC_GLOBAL(a);
}

bool LocalAddress(const IN6_ADDR* addr) {
    return IN6_IS_ADDR_LOOPBACK(addr)         // ::1/128
           || IN6_IS_ADDR_LINKLOCAL(addr)     // fe80::/10
           || IN6_IS_ADDR_SITELOCAL(addr)     // fec0::/10
           || IN6_IS_ADDR_ULA(addr)           // fd00::/8
           || IN6_IS_ADDR_MC_NON_GLOBAL(addr) // ff00::/8 && !(ffxe::/16)
        ;
}

//
// RewriteConnection()
//
// See comment on CalloutClassifyConnect().
//
void RewriteConnection(
    CONTEXT* Context,
    const FWPS_INCOMING_VALUES0* FixedValues,
    const FWPS_INCOMING_METADATA_VALUES0* MetaValues,
    UINT64 FilterId,
    void const* ClassifyContext,
    FWPS_CLASSIFY_OUT0* ClassifyOut) {
    UNREFERENCED_PARAMETER(MetaValues);

    WdfSpinLockAcquire(Context->IpAddresses.Lock);

    auto const ipAddresses = Context->IpAddresses.Addresses;

    WdfSpinLockRelease(Context->IpAddresses.Lock);

    //
    // Identify the specific cases we're interested in or abort.
    //

    bool const ipv4 = FixedValues->layerId == FWPS_LAYER_ALE_CONNECT_REDIRECT_V4;

    if (ipv4) {
        auto const rawLocalAddress = RtlUlongByteSwap(
            FixedValues->incomingValue[FWPS_FIELD_ALE_CONNECT_REDIRECT_V4_IP_LOCAL_ADDRESS].value.uint32);

        auto const rawRemoteAddress = RtlUlongByteSwap(
            FixedValues->incomingValue[FWPS_FIELD_ALE_CONNECT_REDIRECT_V4_IP_REMOTE_ADDRESS].value.uint32);

        auto localAddress = reinterpret_cast<const IN_ADDR*>(&rawLocalAddress);
        auto remoteAddress = reinterpret_cast<const IN_ADDR*>(&rawRemoteAddress);

        auto const shouldRedirect = IN4_ADDR_EQUAL(localAddress, &ipAddresses.TunnelIpv4) ||
                                    !LocalAddress(remoteAddress);

        auto const localPort = FixedValues->incomingValue[FWPS_FIELD_ALE_CONNECT_REDIRECT_V4_IP_LOCAL_PORT].value.uint16;

        auto const remotePort =
            FixedValues->incomingValue[FWPS_FIELD_ALE_CONNECT_REDIRECT_V4_IP_REMOTE_PORT].value.uint16;

        if (!shouldRedirect) {
            LogConnectRedirectPass(HANDLE(MetaValues->processId), localAddress, localPort, remoteAddress, remotePort);

            return;
        }

        LogConnectRedirect(
            HANDLE(MetaValues->processId), localAddress, localPort, &ipAddresses.InternetIpv4, remoteAddress, remotePort);
    } else {
        auto localAddress = reinterpret_cast<const IN6_ADDR*>(
            FixedValues->incomingValue[FWPS_FIELD_ALE_CONNECT_REDIRECT_V6_IP_LOCAL_ADDRESS].value.byteArray16);

        auto remoteAddress = reinterpret_cast<const IN6_ADDR*>(
            FixedValues->incomingValue[FWPS_FIELD_ALE_CONNECT_REDIRECT_V6_IP_REMOTE_ADDRESS].value.byteArray16);

        auto const shouldRedirect = IN6_ADDR_EQUAL(localAddress, &ipAddresses.TunnelIpv6) ||
                                    !LocalAddress(remoteAddress);

        auto const localPort = FixedValues->incomingValue[FWPS_FIELD_ALE_CONNECT_REDIRECT_V6_IP_LOCAL_PORT].value.uint16;

        auto const remotePort =
            FixedValues->incomingValue[FWPS_FIELD_ALE_CONNECT_REDIRECT_V6_IP_REMOTE_PORT].value.uint16;

        if (!shouldRedirect) {
            LogConnectRedirectPass(HANDLE(MetaValues->processId), localAddress, localPort, remoteAddress, remotePort);

            return;
        }

        LogConnectRedirect(
            HANDLE(MetaValues->processId), localAddress, localPort, &ipAddresses.InternetIpv6, remoteAddress, remotePort);
    }

    //
    // Patch local address to force connection off of tunnel interface.
    //

    UINT64 classifyHandle = 0;

    auto status = FwpsAcquireClassifyHandle0(const_cast<void*>(ClassifyContext), 0, &classifyHandle);

    if (!NT_SUCCESS(status)) {
        DbgPrint("FwpsAcquireClassifyHandle0() failed 0x%X\n", status);

        return;
    }

    FWPS_CONNECT_REQUEST0* connectRequest = NULL;

    status = FwpsAcquireWritableLayerDataPointer0(classifyHandle, FilterId, 0, (PVOID*)&connectRequest, ClassifyOut);

    if (!NT_SUCCESS(status)) {
        DbgPrint("FwpsAcquireWritableLayerDataPointer0() failed 0x%X\n", status);
        FwpsReleaseClassifyHandle0(classifyHandle);

        return;
    }

    ClassificationReset(ClassifyOut);

    //
    // There's a list with redirection history.
    //
    // This only ever comes into play if several callouts are fighting to redirect
    // the connection.
    //
    // To prevent recursion, we need to check if we're on the list, and abort if
    // so.
    //

    bool alreadyRedirectedByUs = false;

    for (auto history = connectRequest->previousVersion; history != NULL; history = history->previousVersion) {
        if (history->modifierFilterId == FilterId) {
            DbgPrint("Aborting connection processing because already redirected by us\n");

            alreadyRedirectedByUs = true;
            break;
        }
    }

    if (!alreadyRedirectedByUs) {
        //
        // Rewrite connection.
        //

        if (ipv4) {
            auto localDetails = (SOCKADDR_IN*)&connectRequest->localAddressAndPort;
            localDetails->sin_addr = ipAddresses.InternetIpv4;
        } else {
            auto localDetails = (SOCKADDR_IN6*)&connectRequest->localAddressAndPort;
            localDetails->sin6_addr = ipAddresses.InternetIpv6;
        }

        ClassificationApplySoftPermit(ClassifyOut);
    }

    FwpsApplyModifiedLayerData0(classifyHandle, connectRequest, 0);

    FwpsReleaseClassifyHandle0(classifyHandle);
}

//
// CalloutClassifyConnect()
//
// Adjust properties on new TCP connections.
//
// If an app is marked for splitting, and if a new connection is explicitly made
// on the tunnel interface, or can be assumed to be routed through the tunnel
// interface, then move the connection to the Internet connected interface (LAN
// interface usually).
//
// FWPS_LAYER_ALE_CONNECT_REDIRECT_V4
// FWPS_LAYER_ALE_CONNECT_REDIRECT_V6
//
void CalloutClassifyConnect(
    const FWPS_INCOMING_VALUES0* FixedValues,
    const FWPS_INCOMING_METADATA_VALUES0* MetaValues,
    void* LayerData,
    void const* ClassifyContext,
    const FWPS_FILTER1* Filter,
    UINT64 FlowContext,
    FWPS_CLASSIFY_OUT0* ClassifyOut) {
    UNREFERENCED_PARAMETER(LayerData);
    UNREFERENCED_PARAMETER(FlowContext);

    NT_ASSERT(
        (FixedValues->layerId == FWPS_LAYER_ALE_CONNECT_REDIRECT_V4 &&
         FixedValues->incomingValue[FWPS_FIELD_ALE_CONNECT_REDIRECT_V4_IP_PROTOCOL].value.uint8 == IPPROTO_TCP) ||
        (FixedValues->layerId == FWPS_LAYER_ALE_CONNECT_REDIRECT_V6 &&
         FixedValues->incomingValue[FWPS_FIELD_ALE_CONNECT_REDIRECT_V6_IP_PROTOCOL].value.uint8 == IPPROTO_TCP));

    NT_ASSERT(
        Filter->providerContext != NULL && Filter->providerContext->type == FWPM_GENERAL_CONTEXT &&
        Filter->providerContext->dataBuffer->size == sizeof(CONTEXT*));

    auto context = *(CONTEXT**)Filter->providerContext->dataBuffer->data;

    if ((ClassifyOut->rights & FWPS_RIGHT_ACTION_WRITE) == 0) {
        DbgPrint(
            "Aborting connect-redirect processing because hard permit/block "
            "already applied\n");

        return;
    }

    ClassificationReset(ClassifyOut);

    if (!FWPS_IS_METADATA_FIELD_PRESENT(MetaValues, FWPS_METADATA_FIELD_PROCESS_ID)) {
        DbgPrint("Failed to classify connection because PID was not provided\n");

        return;
    }

    const CALLBACKS& callbacks = context->Callbacks;

    auto const verdict = callbacks.QueryProcess(HANDLE(MetaValues->processId), callbacks.Context);

    switch (verdict) {
    case PROCESS_SPLIT_VERDICT::DO_SPLIT: {
        RewriteConnection(context, FixedValues, MetaValues, Filter->filterId, ClassifyContext, ClassifyOut);

        break;
    }
    case PROCESS_SPLIT_VERDICT::BYPASS: {
        //
        // The process connects using whichever source IP it has bound to.
        // No rewriting; just let the connection proceed.
        //
        ClassificationApplySoftPermit(ClassifyOut);

        break;
    }
    case PROCESS_SPLIT_VERDICT::UNKNOWN: {
        PendClassification(
            context->PendedClassifications,
            HANDLE(MetaValues->processId),
            Filter->filterId,
            FixedValues->layerId,
            ClassifyContext,
            ClassifyOut);

        break;
    }
    };
}

bool IsAleReauthorize(const FWPS_INCOMING_VALUES* FixedValues) {
    size_t index;

    switch (FixedValues->layerId) {
    case FWPS_LAYER_ALE_AUTH_CONNECT_V4: {
        index = FWPS_FIELD_ALE_AUTH_CONNECT_V4_FLAGS;
        break;
    }
    case FWPS_LAYER_ALE_AUTH_RECV_ACCEPT_V4: {
        index = FWPS_FIELD_ALE_AUTH_RECV_ACCEPT_V4_FLAGS;
        break;
    }
    case FWPS_LAYER_ALE_AUTH_CONNECT_V6: {
        index = FWPS_FIELD_ALE_AUTH_CONNECT_V6_FLAGS;
        break;
    }
    case FWPS_LAYER_ALE_AUTH_RECV_ACCEPT_V6: {
        index = FWPS_FIELD_ALE_AUTH_RECV_ACCEPT_V6_FLAGS;
        break;
    }
    default: {
        return false;
    }
    };

    auto const flags = FixedValues->incomingValue[index].value.uint32;

    return ((flags & FWP_CONDITION_FLAG_IS_REAUTHORIZE) != 0);
}

struct AuthLayerValueIndices {
    SIZE_T LocalAddress;
    SIZE_T LocalPort;
    SIZE_T RemoteAddress;
    SIZE_T RemotePort;
};

bool GetAuthLayerValueIndices(UINT16 LayerId, AuthLayerValueIndices* Indices) {
    switch (LayerId) {
    case FWPS_LAYER_ALE_AUTH_CONNECT_V4: {
        *Indices = { FWPS_FIELD_ALE_AUTH_CONNECT_V4_IP_LOCAL_ADDRESS,
                     FWPS_FIELD_ALE_AUTH_CONNECT_V4_IP_LOCAL_PORT,
                     FWPS_FIELD_ALE_AUTH_CONNECT_V4_IP_REMOTE_ADDRESS,
                     FWPS_FIELD_ALE_AUTH_CONNECT_V4_IP_REMOTE_PORT };

        return true;
    }
    case FWPS_LAYER_ALE_AUTH_RECV_ACCEPT_V4: {
        *Indices = { FWPS_FIELD_ALE_AUTH_RECV_ACCEPT_V4_IP_LOCAL_ADDRESS,
                     FWPS_FIELD_ALE_AUTH_RECV_ACCEPT_V4_IP_LOCAL_PORT,
                     FWPS_FIELD_ALE_AUTH_RECV_ACCEPT_V4_IP_REMOTE_ADDRESS,
                     FWPS_FIELD_ALE_AUTH_RECV_ACCEPT_V4_IP_REMOTE_PORT };

        return true;
    }
    case FWPS_LAYER_ALE_AUTH_CONNECT_V6: {
        *Indices = { FWPS_FIELD_ALE_AUTH_CONNECT_V6_IP_LOCAL_ADDRESS,
                     FWPS_FIELD_ALE_AUTH_CONNECT_V6_IP_LOCAL_PORT,
                     FWPS_FIELD_ALE_AUTH_CONNECT_V6_IP_REMOTE_ADDRESS,
                     FWPS_FIELD_ALE_AUTH_CONNECT_V6_IP_REMOTE_PORT };

        return true;
    }
    case FWPS_LAYER_ALE_AUTH_RECV_ACCEPT_V6: {
        *Indices = { FWPS_FIELD_ALE_AUTH_RECV_ACCEPT_V6_IP_LOCAL_ADDRESS,
                     FWPS_FIELD_ALE_AUTH_RECV_ACCEPT_V6_IP_LOCAL_PORT,
                     FWPS_FIELD_ALE_AUTH_RECV_ACCEPT_V6_IP_REMOTE_ADDRESS,
                     FWPS_FIELD_ALE_AUTH_RECV_ACCEPT_V6_IP_REMOTE_PORT };

        return true;
    }
    }

    return false;
}

//
// CalloutPermitSplitApps()
//
// For processes being split, binds and connections will have already been aptly
// redirected. So now it's only a matter of approving the connection.
//
// The reason we have to explicitly approve these connections is because
// otherwise the default filters with lower weights would block all non-tunnel
// connections.
//
// FWPS_LAYER_ALE_AUTH_CONNECT_V4
// FWPS_LAYER_ALE_AUTH_CONNECT_V6
// FWPS_LAYER_ALE_AUTH_RECV_ACCEPT_V4
// FWPS_LAYER_ALE_AUTH_RECV_ACCEPT_V6
//
void CalloutPermitSplitApps(
    const FWPS_INCOMING_VALUES0* FixedValues,
    const FWPS_INCOMING_METADATA_VALUES0* MetaValues,
    void* LayerData,
    void const* ClassifyContext,
    const FWPS_FILTER1* Filter,
    UINT64 FlowContext,
    FWPS_CLASSIFY_OUT0* ClassifyOut) {
    UNREFERENCED_PARAMETER(LayerData);
    UNREFERENCED_PARAMETER(ClassifyContext);
    UNREFERENCED_PARAMETER(FlowContext);

    NT_ASSERT(
        FixedValues->layerId == FWPS_LAYER_ALE_AUTH_CONNECT_V4 ||
        FixedValues->layerId == FWPS_LAYER_ALE_AUTH_CONNECT_V6 ||
        FixedValues->layerId == FWPS_LAYER_ALE_AUTH_RECV_ACCEPT_V4 ||
        FixedValues->layerId == FWPS_LAYER_ALE_AUTH_RECV_ACCEPT_V6);

    NT_ASSERT(
        Filter->providerContext != NULL && Filter->providerContext->type == FWPM_GENERAL_CONTEXT &&
        Filter->providerContext->dataBuffer->size == sizeof(CONTEXT*));

    auto context = *(CONTEXT**)Filter->providerContext->dataBuffer->data;

    if ((ClassifyOut->rights & FWPS_RIGHT_ACTION_WRITE) == 0) {
        DbgPrint("Aborting auth processing because hard permit/block already applied\n");

        return;
    }

    ClassificationReset(ClassifyOut);

    if (!FWPS_IS_METADATA_FIELD_PRESENT(MetaValues, FWPS_METADATA_FIELD_PROCESS_ID)) {
        DbgPrint("Failed to complete auth processing because PID was not provided\n");

        return;
    }

    const CALLBACKS& callbacks = context->Callbacks;

    auto const verdict = callbacks.QueryProcess(HANDLE(MetaValues->processId), callbacks.Context);

    //
    // If the process is not marked for splitting or bypass we should just abort
    // and not attempt to classify the connection.
    //

    if (verdict != PROCESS_SPLIT_VERDICT::DO_SPLIT && verdict != PROCESS_SPLIT_VERDICT::BYPASS) {
        return;
    }

    //
    // Include extensive logging.
    //

    AuthLayerValueIndices indices = { 0 };

    auto const status = GetAuthLayerValueIndices(FixedValues->layerId, &indices);

    NT_ASSERT(status);

    if (!status) {
        return;
    }

    auto const localPort = FixedValues->incomingValue[indices.LocalPort].value.uint16;
    auto const remotePort = FixedValues->incomingValue[indices.RemotePort].value.uint16;

    bool const ipv4 = FixedValues->layerId == FWPS_LAYER_ALE_AUTH_CONNECT_V4 ||
                      FixedValues->layerId == FWPS_LAYER_ALE_AUTH_RECV_ACCEPT_V4;

    if (ipv4) {
        auto const rawLocalAddress = RtlUlongByteSwap(FixedValues->incomingValue[indices.LocalAddress].value.uint32);

        auto const rawRemoteAddress = RtlUlongByteSwap(FixedValues->incomingValue[indices.RemoteAddress].value.uint32);

        auto localAddress = reinterpret_cast<const IN_ADDR*>(&rawLocalAddress);
        auto remoteAddress = reinterpret_cast<const IN_ADDR*>(&rawRemoteAddress);

        LogPermitConnection(
            HANDLE(MetaValues->processId),
            localAddress,
            localPort,
            remoteAddress,
            remotePort,
            (FixedValues->layerId == FWPS_LAYER_ALE_AUTH_CONNECT_V4));
    } else {
        auto localAddress =
            reinterpret_cast<const IN6_ADDR*>(FixedValues->incomingValue[indices.LocalAddress].value.byteArray16);

        auto remoteAddress =
            reinterpret_cast<const IN6_ADDR*>(FixedValues->incomingValue[indices.RemoteAddress].value.byteArray16);

        LogPermitConnection(
            HANDLE(MetaValues->processId),
            localAddress,
            localPort,
            remoteAddress,
            remotePort,
            (FixedValues->layerId == FWPS_LAYER_ALE_AUTH_CONNECT_V6));
    }

    //
    // Apply classification.
    //

    ClassificationApplySoftPermit(ClassifyOut);
}

//
// CalloutBlockSplitApps()
//
// For processes just now being split, it could be the case that they have
// existing long-lived connections inside the tunnel.
//
// These connections need to be blocked to ensure the process exists on
// only one side of the tunnel.
//
// Additionally, block any processes which have not been evaluated.
//
// This normally isn't required because earlier callouts re-auth until the
// process is categorized, but it makes sense as a safety measure.
//
// FWPS_LAYER_ALE_AUTH_CONNECT_V4
// FWPS_LAYER_ALE_AUTH_CONNECT_V6
// FWPS_LAYER_ALE_AUTH_RECV_ACCEPT_V4
// FWPS_LAYER_ALE_AUTH_RECV_ACCEPT_V6
//
void CalloutBlockSplitApps(
    const FWPS_INCOMING_VALUES0* FixedValues,
    const FWPS_INCOMING_METADATA_VALUES0* MetaValues,
    void* LayerData,
    void const* ClassifyContext,
    const FWPS_FILTER1* Filter,
    UINT64 FlowContext,
    FWPS_CLASSIFY_OUT0* ClassifyOut) {
    UNREFERENCED_PARAMETER(LayerData);
    UNREFERENCED_PARAMETER(ClassifyContext);
    UNREFERENCED_PARAMETER(FlowContext);

    NT_ASSERT(
        FixedValues->layerId == FWPS_LAYER_ALE_AUTH_CONNECT_V4 ||
        FixedValues->layerId == FWPS_LAYER_ALE_AUTH_CONNECT_V6 ||
        FixedValues->layerId == FWPS_LAYER_ALE_AUTH_RECV_ACCEPT_V4 ||
        FixedValues->layerId == FWPS_LAYER_ALE_AUTH_RECV_ACCEPT_V6);

    NT_ASSERT(
        Filter->providerContext != NULL && Filter->providerContext->type == FWPM_GENERAL_CONTEXT &&
        Filter->providerContext->dataBuffer->size == sizeof(CONTEXT*));

    auto context = *(CONTEXT**)Filter->providerContext->dataBuffer->data;

    if ((ClassifyOut->rights & FWPS_RIGHT_ACTION_WRITE) == 0) {
        DbgPrint("Aborting auth processing because hard permit/block already applied\n");

        return;
    }

    ClassificationReset(ClassifyOut);

    if (!FWPS_IS_METADATA_FIELD_PRESENT(MetaValues, FWPS_METADATA_FIELD_PROCESS_ID)) {
        DbgPrint("Failed to complete auth processing because PID was not provided\n");

        return;
    }

    const CALLBACKS& callbacks = context->Callbacks;

    auto const verdict = callbacks.QueryProcess(HANDLE(MetaValues->processId), callbacks.Context);

    //
    // Block any processes which have not yet been evaluated.
    // This is a safety measure to prevent race conditions.
    //
    // Do not block bypass processes; their traffic is routed by source IP.
    //

    auto const shouldBlock = (verdict == PROCESS_SPLIT_VERDICT::DO_SPLIT) || (verdict == PROCESS_SPLIT_VERDICT::UNKNOWN);

    if (!shouldBlock) {
        return;
    }

    //
    // Include extensive logging.
    //

    AuthLayerValueIndices indices = { 0 };

    auto const status = GetAuthLayerValueIndices(FixedValues->layerId, &indices);

    NT_ASSERT(status);

    if (!status) {
        return;
    }

    auto const localPort = FixedValues->incomingValue[indices.LocalPort].value.uint16;
    auto const remotePort = FixedValues->incomingValue[indices.RemotePort].value.uint16;

    bool const ipv4 = FixedValues->layerId == FWPS_LAYER_ALE_AUTH_CONNECT_V4 ||
                      FixedValues->layerId == FWPS_LAYER_ALE_AUTH_RECV_ACCEPT_V4;

    if (ipv4) {
        auto const rawLocalAddress = RtlUlongByteSwap(FixedValues->incomingValue[indices.LocalAddress].value.uint32);

        auto const rawRemoteAddress = RtlUlongByteSwap(FixedValues->incomingValue[indices.RemoteAddress].value.uint32);

        auto localAddress = reinterpret_cast<const IN_ADDR*>(&rawLocalAddress);
        auto remoteAddress = reinterpret_cast<const IN_ADDR*>(&rawRemoteAddress);

        LogBlockConnection(
            HANDLE(MetaValues->processId),
            localAddress,
            localPort,
            remoteAddress,
            remotePort,
            (FixedValues->layerId == FWPS_LAYER_ALE_AUTH_CONNECT_V4));
    } else {
        auto localAddress =
            reinterpret_cast<const IN6_ADDR*>(FixedValues->incomingValue[indices.LocalAddress].value.byteArray16);

        auto remoteAddress =
            reinterpret_cast<const IN6_ADDR*>(FixedValues->incomingValue[indices.RemoteAddress].value.byteArray16);

        LogBlockConnection(
            HANDLE(MetaValues->processId),
            localAddress,
            localPort,
            remoteAddress,
            remotePort,
            (FixedValues->layerId == FWPS_LAYER_ALE_AUTH_CONNECT_V6));
    }

    //
    // Apply classification.
    //

    ClassificationApplyHardBlock(ClassifyOut);
}

} // anonymous namespace

//
// RegisterCalloutClassifyBindTx()
//
// Register callout with WFP. In all applicable layers.
//
// "Tx" (in transaction) suffix means there is no clean-up in failure paths.
//
NTSTATUS
RegisterCalloutClassifyBindTx(PDEVICE_OBJECT DeviceObject, HANDLE WfpSession) {
    auto status = RegisterCalloutTx(
        DeviceObject,
        WfpSession,
        CalloutClassifyBind,
        &ST_FW_CALLOUT_CLASSIFY_BIND_IPV4_KEY,
        &FWPM_LAYER_ALE_BIND_REDIRECT_V4,
        L"Nym VPN Split Tunnel Bind Redirect Callout (IPv4)",
        L"Redirects certain binds away from tunnel interface");

    if (!NT_SUCCESS(status)) {
        return status;
    }

    status = RegisterCalloutTx(
        DeviceObject,
        WfpSession,
        CalloutClassifyBind,
        &ST_FW_CALLOUT_CLASSIFY_BIND_IPV6_KEY,
        &FWPM_LAYER_ALE_BIND_REDIRECT_V6,
        L"Nym VPN Split Tunnel Bind Redirect Callout (IPv6)",
        L"Redirects certain binds away from tunnel interface");

    if (!NT_SUCCESS(status)) {
        UnregisterCalloutClassifyBind();
    }

    return status;
}

NTSTATUS
UnregisterCalloutClassifyBind() {
    auto s1 = UnregisterCallout(&ST_FW_CALLOUT_CLASSIFY_BIND_IPV4_KEY);
    auto s2 = UnregisterCallout(&ST_FW_CALLOUT_CLASSIFY_BIND_IPV6_KEY);

    RETURN_IF_UNSUCCESSFUL(s1);
    RETURN_IF_UNSUCCESSFUL(s2);

    return STATUS_SUCCESS;
}

//
// RegisterCalloutClassifyConnectTx()
//
// Register callout with WFP. In all applicable layers.
//
// "Tx" (in transaction) suffix means there is no clean-up in failure paths.
//
NTSTATUS
RegisterCalloutClassifyConnectTx(PDEVICE_OBJECT DeviceObject, HANDLE WfpSession) {
    auto status = RegisterCalloutTx(
        DeviceObject,
        WfpSession,
        CalloutClassifyConnect,
        &ST_FW_CALLOUT_CLASSIFY_CONNECT_IPV4_KEY,
        &FWPM_LAYER_ALE_CONNECT_REDIRECT_V4,
        L"Nym VPN Split Tunnel Connect Redirect Callout (IPv4)",
        L"Adjusts properties on new network connections");

    if (!NT_SUCCESS(status)) {
        return status;
    }

    status = RegisterCalloutTx(
        DeviceObject,
        WfpSession,
        CalloutClassifyConnect,
        &ST_FW_CALLOUT_CLASSIFY_CONNECT_IPV6_KEY,
        &FWPM_LAYER_ALE_CONNECT_REDIRECT_V6,
        L"Nym VPN Split Tunnel Connect Redirect Callout (IPv6)",
        L"Adjusts properties on new network connections");

    if (!NT_SUCCESS(status)) {
        UnregisterCalloutClassifyConnect();
    }

    return status;
}

NTSTATUS
UnregisterCalloutClassifyConnect() {
    auto s1 = UnregisterCallout(&ST_FW_CALLOUT_CLASSIFY_CONNECT_IPV4_KEY);
    auto s2 = UnregisterCallout(&ST_FW_CALLOUT_CLASSIFY_CONNECT_IPV6_KEY);

    RETURN_IF_UNSUCCESSFUL(s1);
    RETURN_IF_UNSUCCESSFUL(s2);

    return STATUS_SUCCESS;
}

//
// RegisterCalloutPermitSplitAppsTx()
//
// Register callout with WFP. In all applicable layers.
//
// "Tx" (in transaction) suffix means there is no clean-up in failure paths.
//
NTSTATUS
RegisterCalloutPermitSplitAppsTx(PDEVICE_OBJECT DeviceObject, HANDLE WfpSession) {
    auto status = RegisterCalloutTx(
        DeviceObject,
        WfpSession,
        CalloutPermitSplitApps,
        &ST_FW_CALLOUT_PERMIT_SPLIT_APPS_IPV4_CONN_KEY,
        &FWPM_LAYER_ALE_AUTH_CONNECT_V4,
        L"Nym VPN Split Tunnel Permitting Callout (IPv4)",
        L"Permits selected connections outside the tunnel");

    if (!NT_SUCCESS(status)) {
        return status;
    }

    status = RegisterCalloutTx(
        DeviceObject,
        WfpSession,
        CalloutPermitSplitApps,
        &ST_FW_CALLOUT_PERMIT_SPLIT_APPS_IPV4_RECV_KEY,
        &FWPM_LAYER_ALE_AUTH_RECV_ACCEPT_V4,
        L"Nym VPN Split Tunnel Permitting Callout (IPv4)",
        L"Permits selected connections outside the tunnel");

    if (!NT_SUCCESS(status)) {
        UnregisterCalloutPermitSplitApps();

        return status;
    }

    status = RegisterCalloutTx(
        DeviceObject,
        WfpSession,
        CalloutPermitSplitApps,
        &ST_FW_CALLOUT_PERMIT_SPLIT_APPS_IPV6_CONN_KEY,
        &FWPM_LAYER_ALE_AUTH_CONNECT_V6,
        L"Nym VPN Split Tunnel Permitting Callout (IPv6)",
        L"Permits selected connections outside the tunnel");

    if (!NT_SUCCESS(status)) {
        UnregisterCalloutPermitSplitApps();

        return status;
    }

    status = RegisterCalloutTx(
        DeviceObject,
        WfpSession,
        CalloutPermitSplitApps,
        &ST_FW_CALLOUT_PERMIT_SPLIT_APPS_IPV6_RECV_KEY,
        &FWPM_LAYER_ALE_AUTH_RECV_ACCEPT_V6,
        L"Nym VPN Split Tunnel Permitting Callout (IPv6)",
        L"Permits selected connections outside the tunnel");

    if (!NT_SUCCESS(status)) {
        UnregisterCalloutPermitSplitApps();

        return status;
    }

    return STATUS_SUCCESS;
}

NTSTATUS
UnregisterCalloutPermitSplitApps() {
    auto s1 = UnregisterCallout(&ST_FW_CALLOUT_PERMIT_SPLIT_APPS_IPV4_CONN_KEY);
    auto s2 = UnregisterCallout(&ST_FW_CALLOUT_PERMIT_SPLIT_APPS_IPV4_RECV_KEY);
    auto s3 = UnregisterCallout(&ST_FW_CALLOUT_PERMIT_SPLIT_APPS_IPV6_CONN_KEY);
    auto s4 = UnregisterCallout(&ST_FW_CALLOUT_PERMIT_SPLIT_APPS_IPV6_RECV_KEY);

    RETURN_IF_UNSUCCESSFUL(s1);
    RETURN_IF_UNSUCCESSFUL(s2);
    RETURN_IF_UNSUCCESSFUL(s3);
    RETURN_IF_UNSUCCESSFUL(s4);

    return STATUS_SUCCESS;
}

//
// RegisterCalloutBlockSplitAppsTx()
//
// Register callout with WFP. In all applicable layers.
//
// "Tx" (in transaction) suffix means there is no clean-up in failure paths.
//
NTSTATUS
RegisterCalloutBlockSplitAppsTx(PDEVICE_OBJECT DeviceObject, HANDLE WfpSession) {
    auto status = RegisterCalloutTx(
        DeviceObject,
        WfpSession,
        CalloutBlockSplitApps,
        &ST_FW_CALLOUT_BLOCK_SPLIT_APPS_IPV4_CONN_KEY,
        &FWPM_LAYER_ALE_AUTH_CONNECT_V4,
        L"Nym VPN Split Tunnel Blocking Callout (IPv4)",
        L"Blocks unwanted connections in relation to splitting");

    if (!NT_SUCCESS(status)) {
        return status;
    }

    status = RegisterCalloutTx(
        DeviceObject,
        WfpSession,
        CalloutBlockSplitApps,
        &ST_FW_CALLOUT_BLOCK_SPLIT_APPS_IPV4_RECV_KEY,
        &FWPM_LAYER_ALE_AUTH_RECV_ACCEPT_V4,
        L"Nym VPN Split Tunnel Blocking Callout (IPv4)",
        L"Blocks unwanted connections in relation to splitting");

    if (!NT_SUCCESS(status)) {
        UnregisterCalloutBlockSplitApps();

        return status;
    }

    status = RegisterCalloutTx(
        DeviceObject,
        WfpSession,
        CalloutBlockSplitApps,
        &ST_FW_CALLOUT_BLOCK_SPLIT_APPS_IPV6_CONN_KEY,
        &FWPM_LAYER_ALE_AUTH_CONNECT_V6,
        L"Nym VPN Split Tunnel Blocking Callout (IPv6)",
        L"Blocks unwanted connections in relation to splitting");

    if (!NT_SUCCESS(status)) {
        UnregisterCalloutBlockSplitApps();

        return status;
    }

    status = RegisterCalloutTx(
        DeviceObject,
        WfpSession,
        CalloutBlockSplitApps,
        &ST_FW_CALLOUT_BLOCK_SPLIT_APPS_IPV6_RECV_KEY,
        &FWPM_LAYER_ALE_AUTH_RECV_ACCEPT_V6,
        L"Nym VPN Split Tunnel Blocking Callout (IPv6)",
        L"Blocks unwanted connections in relation to splitting");

    if (!NT_SUCCESS(status)) {
        UnregisterCalloutBlockSplitApps();

        return status;
    }

    return STATUS_SUCCESS;
}

NTSTATUS
UnregisterCalloutBlockSplitApps() {
    auto s1 = UnregisterCallout(&ST_FW_CALLOUT_BLOCK_SPLIT_APPS_IPV4_CONN_KEY);
    auto s2 = UnregisterCallout(&ST_FW_CALLOUT_BLOCK_SPLIT_APPS_IPV4_RECV_KEY);
    auto s3 = UnregisterCallout(&ST_FW_CALLOUT_BLOCK_SPLIT_APPS_IPV6_CONN_KEY);
    auto s4 = UnregisterCallout(&ST_FW_CALLOUT_BLOCK_SPLIT_APPS_IPV6_RECV_KEY);

    RETURN_IF_UNSUCCESSFUL(s1);
    RETURN_IF_UNSUCCESSFUL(s2);
    RETURN_IF_UNSUCCESSFUL(s3);
    RETURN_IF_UNSUCCESSFUL(s4);

    return STATUS_SUCCESS;
}

} // namespace firewall
