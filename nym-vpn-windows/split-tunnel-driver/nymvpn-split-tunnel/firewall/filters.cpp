// Copyright 2016-2026 Mullvad VPN AB. All Rights Reserved.
// Copyright 2026 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

#include "filters.h"
#include "../util.h"
#include "constants.h"
#include "identifiers.h"

#define SUCCEED_OR_RETURN(status)                                                                                      \
    if (!NT_SUCCESS(status)) {                                                                                         \
        return status;                                                                                                 \
    }

namespace firewall {

NTSTATUS
RegisterFilterBindRedirectIpv4Tx(HANDLE WfpSession) {
    //
    // Create filter that references callout.
    //
    // Use `protocol != TCP` as the sole condition.
    // This is because TCP traffic is better dealt with in connect-redirect.
    //

    FWPM_FILTER0 filter = { 0 };

    auto const filterName = L"Nym VPN Split Tunnel Bind Redirect Filter (IPv4)";
    auto const filterDescription = L"Redirects certain binds away from tunnel interface";

    filter.filterKey = ST_FW_FILTER_CLASSIFY_BIND_IPV4_KEY;
    filter.displayData.name = const_cast<wchar_t*>(filterName);
    filter.displayData.description = const_cast<wchar_t*>(filterDescription);
    filter.flags = FWPM_FILTER_FLAG_CLEAR_ACTION_RIGHT | FWPM_FILTER_FLAG_HAS_PROVIDER_CONTEXT;
    filter.providerKey = const_cast<GUID*>(&ST_FW_PROVIDER_KEY);
    filter.layerKey = FWPM_LAYER_ALE_BIND_REDIRECT_V4;
    filter.subLayerKey = ST_FW_WINFW_BASELINE_SUBLAYER_KEY;
    filter.weight.type = FWP_UINT64;
    filter.weight.uint64 = const_cast<UINT64*>(&ST_MAX_FILTER_WEIGHT);
    filter.action.type = FWP_ACTION_CALLOUT_UNKNOWN;
    filter.action.calloutKey = ST_FW_CALLOUT_CLASSIFY_BIND_IPV4_KEY;
    filter.providerContextKey = ST_FW_PROVIDER_CONTEXT_KEY;

    FWPM_FILTER_CONDITION0 cond;

    cond.fieldKey = FWPM_CONDITION_IP_PROTOCOL;
    cond.matchType = FWP_MATCH_NOT_EQUAL;
    cond.conditionValue.type = FWP_UINT8;
    cond.conditionValue.uint8 = IPPROTO_TCP;

    filter.filterCondition = &cond;
    filter.numFilterConditions = 1;

    return FwpmFilterAdd0(WfpSession, &filter, NULL, NULL);
}

NTSTATUS
RemoveFilterBindRedirectIpv4Tx(HANDLE WfpSession) {
    return FwpmFilterDeleteByKey0(WfpSession, &ST_FW_FILTER_CLASSIFY_BIND_IPV4_KEY);
}

NTSTATUS
RegisterFilterBindRedirectIpv6Tx(HANDLE WfpSession) {
    //
    // Create filter that references callout.
    //
    // Use `protocol != TCP` as the sole condition.
    // This is because TCP traffic is better dealt with in connect-redirect.
    //

    FWPM_FILTER0 filter = { 0 };

    auto const filterName = L"Nym VPN Split Tunnel Bind Redirect Filter (IPv6)";
    auto const filterDescription = L"Redirects certain binds away from tunnel interface";

    filter.filterKey = ST_FW_FILTER_CLASSIFY_BIND_IPV6_KEY;
    filter.displayData.name = const_cast<wchar_t*>(filterName);
    filter.displayData.description = const_cast<wchar_t*>(filterDescription);
    filter.flags = FWPM_FILTER_FLAG_CLEAR_ACTION_RIGHT | FWPM_FILTER_FLAG_HAS_PROVIDER_CONTEXT;
    filter.providerKey = const_cast<GUID*>(&ST_FW_PROVIDER_KEY);
    filter.layerKey = FWPM_LAYER_ALE_BIND_REDIRECT_V6;
    filter.subLayerKey = ST_FW_WINFW_BASELINE_SUBLAYER_KEY;
    filter.weight.type = FWP_UINT64;
    filter.weight.uint64 = const_cast<UINT64*>(&ST_MAX_FILTER_WEIGHT);
    filter.action.type = FWP_ACTION_CALLOUT_UNKNOWN;
    filter.action.calloutKey = ST_FW_CALLOUT_CLASSIFY_BIND_IPV6_KEY;
    filter.providerContextKey = ST_FW_PROVIDER_CONTEXT_KEY;

    FWPM_FILTER_CONDITION0 cond;

    cond.fieldKey = FWPM_CONDITION_IP_PROTOCOL;
    cond.matchType = FWP_MATCH_NOT_EQUAL;
    cond.conditionValue.type = FWP_UINT8;
    cond.conditionValue.uint8 = IPPROTO_TCP;

    filter.filterCondition = &cond;
    filter.numFilterConditions = 1;

    return FwpmFilterAdd0(WfpSession, &filter, NULL, NULL);
}

NTSTATUS
RemoveFilterBindRedirectIpv6Tx(HANDLE WfpSession) {
    return FwpmFilterDeleteByKey0(WfpSession, &ST_FW_FILTER_CLASSIFY_BIND_IPV6_KEY);
}

NTSTATUS
RegisterFilterConnectRedirectIpv4Tx(HANDLE WfpSession) {
    //
    // Create filter that references callout.
    //
    // Use `protocol == TCP` as the sole condition.
    //
    // This is because the source address for non-TCP traffic can't be updated in
    // connect-redirect. So that traffic is instead dealt with in bind-redirect.
    //

    FWPM_FILTER0 filter = { 0 };

    auto const filterName = L"Nym VPN Split Tunnel Connect Redirect Filter (IPv4)";
    auto const filterDescription = L"Adjusts properties on new network connections";

    filter.filterKey = ST_FW_FILTER_CLASSIFY_CONNECT_IPV4_KEY;
    filter.displayData.name = const_cast<wchar_t*>(filterName);
    filter.displayData.description = const_cast<wchar_t*>(filterDescription);
    filter.flags = FWPM_FILTER_FLAG_CLEAR_ACTION_RIGHT | FWPM_FILTER_FLAG_HAS_PROVIDER_CONTEXT;
    filter.providerKey = const_cast<GUID*>(&ST_FW_PROVIDER_KEY);
    filter.layerKey = FWPM_LAYER_ALE_CONNECT_REDIRECT_V4;
    filter.subLayerKey = ST_FW_WINFW_BASELINE_SUBLAYER_KEY;
    filter.weight.type = FWP_UINT64;
    filter.weight.uint64 = const_cast<UINT64*>(&ST_MAX_FILTER_WEIGHT);
    filter.action.type = FWP_ACTION_CALLOUT_UNKNOWN;
    filter.action.calloutKey = ST_FW_CALLOUT_CLASSIFY_CONNECT_IPV4_KEY;
    filter.providerContextKey = ST_FW_PROVIDER_CONTEXT_KEY;

    FWPM_FILTER_CONDITION0 cond;

    cond.fieldKey = FWPM_CONDITION_IP_PROTOCOL;
    cond.matchType = FWP_MATCH_EQUAL;
    cond.conditionValue.type = FWP_UINT8;
    cond.conditionValue.uint8 = IPPROTO_TCP;

    filter.filterCondition = &cond;
    filter.numFilterConditions = 1;

    return FwpmFilterAdd0(WfpSession, &filter, NULL, NULL);
}

NTSTATUS
RemoveFilterConnectRedirectIpv4Tx(HANDLE WfpSession) {
    return FwpmFilterDeleteByKey0(WfpSession, &ST_FW_FILTER_CLASSIFY_CONNECT_IPV4_KEY);
}

NTSTATUS
RegisterFilterConnectRedirectIpv6Tx(HANDLE WfpSession) {
    //
    // Create filter that references callout.
    //
    // Use `protocol == TCP` as the sole condition.
    //
    // This is because the source address for non-TCP traffic can't be updated in
    // connect-redirect. So that traffic is instead dealt with in bind-redirect.
    //

    FWPM_FILTER0 filter = { 0 };

    auto const filterName = L"Nym VPN Split Tunnel Connect Redirect Filter (IPv6)";
    auto const filterDescription = L"Adjusts properties on new network connections";

    filter.filterKey = ST_FW_FILTER_CLASSIFY_CONNECT_IPV6_KEY;
    filter.displayData.name = const_cast<wchar_t*>(filterName);
    filter.displayData.description = const_cast<wchar_t*>(filterDescription);
    filter.flags = FWPM_FILTER_FLAG_CLEAR_ACTION_RIGHT | FWPM_FILTER_FLAG_HAS_PROVIDER_CONTEXT;
    filter.providerKey = const_cast<GUID*>(&ST_FW_PROVIDER_KEY);
    filter.layerKey = FWPM_LAYER_ALE_CONNECT_REDIRECT_V6;
    filter.subLayerKey = ST_FW_WINFW_BASELINE_SUBLAYER_KEY;
    filter.weight.type = FWP_UINT64;
    filter.weight.uint64 = const_cast<UINT64*>(&ST_MAX_FILTER_WEIGHT);
    filter.action.type = FWP_ACTION_CALLOUT_UNKNOWN;
    filter.action.calloutKey = ST_FW_CALLOUT_CLASSIFY_CONNECT_IPV6_KEY;
    filter.providerContextKey = ST_FW_PROVIDER_CONTEXT_KEY;

    FWPM_FILTER_CONDITION0 cond;

    cond.fieldKey = FWPM_CONDITION_IP_PROTOCOL;
    cond.matchType = FWP_MATCH_EQUAL;
    cond.conditionValue.type = FWP_UINT8;
    cond.conditionValue.uint8 = IPPROTO_TCP;

    filter.filterCondition = &cond;
    filter.numFilterConditions = 1;

    return FwpmFilterAdd0(WfpSession, &filter, NULL, NULL);
}

NTSTATUS
RemoveFilterConnectRedirectIpv6Tx(HANDLE WfpSession) {
    return FwpmFilterDeleteByKey0(WfpSession, &ST_FW_FILTER_CLASSIFY_CONNECT_IPV6_KEY);
}

NTSTATUS
RegisterFilterPermitNonTunnelIpv4Tx(HANDLE WfpSession, const IN_ADDR* TunnelIpv4) {
    //
    // Create filter that references callout.
    //
    // The single condition is IP_LOCAL_ADDRESS != Tunnel.
    //
    // This ensures the callout is presented only with connections that are
    // attempted outside the tunnel.
    //
    // Ipv4 outbound.
    //

    FWPM_FILTER0 filter = { 0 };

    auto const filterName = L"Nym VPN Split Tunnel Permissive Filter (IPv4)";
    auto const filterDescription = L"Approves selected connections outside the tunnel";

    filter.filterKey = ST_FW_FILTER_PERMIT_SPLIT_APPS_IPV4_CONN_KEY;
    filter.displayData.name = const_cast<wchar_t*>(filterName);
    filter.displayData.description = const_cast<wchar_t*>(filterDescription);
    filter.flags = FWPM_FILTER_FLAG_HAS_PROVIDER_CONTEXT;
    filter.providerKey = const_cast<GUID*>(&ST_FW_PROVIDER_KEY);
    filter.layerKey = FWPM_LAYER_ALE_AUTH_CONNECT_V4;
    filter.subLayerKey = ST_FW_WINFW_BASELINE_SUBLAYER_KEY;
    filter.weight.type = FWP_UINT64;
    filter.weight.uint64 = const_cast<UINT64*>(&ST_HIGH_FILTER_WEIGHT);
    filter.action.type = FWP_ACTION_CALLOUT_UNKNOWN;
    filter.action.calloutKey = ST_FW_CALLOUT_PERMIT_SPLIT_APPS_IPV4_CONN_KEY;
    filter.providerContextKey = ST_FW_PROVIDER_CONTEXT_KEY;

    FWPM_FILTER_CONDITION0 cond = { 0 };

    //
    // If there's no tunnel IPv4 interface then traffic on all interfaces
    // qualifies as non-tunnel traffic.
    //

    if (TunnelIpv4 != NULL) {
        cond.fieldKey = FWPM_CONDITION_IP_LOCAL_ADDRESS;
        cond.matchType = FWP_MATCH_NOT_EQUAL;
        cond.conditionValue.type = FWP_UINT32;
        cond.conditionValue.uint32 = RtlUlongByteSwap(TunnelIpv4->s_addr);

        filter.filterCondition = &cond;
        filter.numFilterConditions = 1;
    }

    SUCCEED_OR_RETURN(FwpmFilterAdd0(WfpSession, &filter, NULL, NULL));

    //
    // Ipv4 inbound.
    //

    filter.filterKey = ST_FW_FILTER_PERMIT_SPLIT_APPS_IPV4_RECV_KEY;
    filter.layerKey = FWPM_LAYER_ALE_AUTH_RECV_ACCEPT_V4;
    filter.action.calloutKey = ST_FW_CALLOUT_PERMIT_SPLIT_APPS_IPV4_RECV_KEY;

    SUCCEED_OR_RETURN(FwpmFilterAdd0(WfpSession, &filter, NULL, NULL));

    //
    // Create corresponding filters in the DNS sublayer.
    // By convention, these filters should include a condition on the destination
    // port.
    //
    // I.e. we'll be using 1 or 2 conditions.
    //

    FWPM_FILTER_CONDITION0 dnscond[2] = { 0, 0 };

    dnscond[0].fieldKey = FWPM_CONDITION_IP_REMOTE_PORT;
    dnscond[0].matchType = FWP_MATCH_EQUAL;
    dnscond[0].conditionValue.type = FWP_UINT16;
    dnscond[0].conditionValue.uint16 = DNS_SERVER_PORT;

    filter.filterCondition = dnscond;

    if (TunnelIpv4 != NULL) {
        dnscond[1] = cond;
        filter.numFilterConditions = 2;
    } else {
        filter.numFilterConditions = 1;
    }

    //
    // Ipv4 outbound DNS.
    //

    filter.filterKey = ST_FW_FILTER_PERMIT_SPLIT_APPS_IPV4_DNS_CONN_KEY;
    filter.layerKey = FWPM_LAYER_ALE_AUTH_CONNECT_V4;
    filter.subLayerKey = ST_FW_WINFW_DNS_SUBLAYER_KEY;
    filter.action.calloutKey = ST_FW_CALLOUT_PERMIT_SPLIT_APPS_IPV4_CONN_KEY;

    SUCCEED_OR_RETURN(FwpmFilterAdd0(WfpSession, &filter, NULL, NULL));

    //
    // Ipv4 inbound DNS.
    //

    filter.filterKey = ST_FW_FILTER_PERMIT_SPLIT_APPS_IPV4_DNS_RECV_KEY;
    filter.layerKey = FWPM_LAYER_ALE_AUTH_RECV_ACCEPT_V4;
    filter.action.calloutKey = ST_FW_CALLOUT_PERMIT_SPLIT_APPS_IPV4_RECV_KEY;

    return FwpmFilterAdd0(WfpSession, &filter, NULL, NULL);
}

NTSTATUS
RemoveFilterPermitNonTunnelIpv4Tx(HANDLE WfpSession) {
    SUCCEED_OR_RETURN(FwpmFilterDeleteByKey0(WfpSession, &ST_FW_FILTER_PERMIT_SPLIT_APPS_IPV4_CONN_KEY));

    SUCCEED_OR_RETURN(FwpmFilterDeleteByKey0(WfpSession, &ST_FW_FILTER_PERMIT_SPLIT_APPS_IPV4_RECV_KEY));

    SUCCEED_OR_RETURN(FwpmFilterDeleteByKey0(WfpSession, &ST_FW_FILTER_PERMIT_SPLIT_APPS_IPV4_DNS_CONN_KEY));

    return FwpmFilterDeleteByKey0(WfpSession, &ST_FW_FILTER_PERMIT_SPLIT_APPS_IPV4_DNS_RECV_KEY);
}

NTSTATUS
RegisterFilterPermitNonTunnelIpv6Tx(HANDLE WfpSession, const IN6_ADDR* TunnelIpv6) {
    //
    // IPv6 outbound.
    //

    FWPM_FILTER0 filter = { 0 };

    auto const filterName = L"Nym VPN Split Tunnel Permissive Filter (IPv6)";
    auto const filterDescription = L"Approves selected connections outside the tunnel";

    filter.filterKey = ST_FW_FILTER_PERMIT_SPLIT_APPS_IPV6_CONN_KEY;
    filter.displayData.name = const_cast<wchar_t*>(filterName);
    filter.displayData.description = const_cast<wchar_t*>(filterDescription);
    filter.flags = FWPM_FILTER_FLAG_HAS_PROVIDER_CONTEXT;
    filter.providerKey = const_cast<GUID*>(&ST_FW_PROVIDER_KEY);
    filter.layerKey = FWPM_LAYER_ALE_AUTH_CONNECT_V6;
    filter.subLayerKey = ST_FW_WINFW_BASELINE_SUBLAYER_KEY;
    filter.weight.type = FWP_UINT64;
    filter.weight.uint64 = const_cast<UINT64*>(&ST_HIGH_FILTER_WEIGHT);
    filter.action.type = FWP_ACTION_CALLOUT_UNKNOWN;
    filter.action.calloutKey = ST_FW_CALLOUT_PERMIT_SPLIT_APPS_IPV6_CONN_KEY;
    filter.providerContextKey = ST_FW_PROVIDER_CONTEXT_KEY;

    FWPM_FILTER_CONDITION0 cond = { 0 };

    //
    // If there's no tunnel IPv6 interface then traffic on all interfaces
    // qualifies as non-tunnel traffic.
    //

    if (TunnelIpv6 != NULL) {
        cond.fieldKey = FWPM_CONDITION_IP_LOCAL_ADDRESS;
        cond.matchType = FWP_MATCH_NOT_EQUAL;
        cond.conditionValue.type = FWP_BYTE_ARRAY16_TYPE;
        cond.conditionValue.byteArray16 = (FWP_BYTE_ARRAY16*)TunnelIpv6->u.Byte;

        filter.filterCondition = &cond;
        filter.numFilterConditions = 1;
    }

    SUCCEED_OR_RETURN(FwpmFilterAdd0(WfpSession, &filter, NULL, NULL));

    //
    // IPv6 inbound.
    //

    filter.filterKey = ST_FW_FILTER_PERMIT_SPLIT_APPS_IPV6_RECV_KEY;
    filter.layerKey = FWPM_LAYER_ALE_AUTH_RECV_ACCEPT_V6;
    filter.action.calloutKey = ST_FW_CALLOUT_PERMIT_SPLIT_APPS_IPV6_RECV_KEY;

    SUCCEED_OR_RETURN(FwpmFilterAdd0(WfpSession, &filter, NULL, NULL));

    //
    // Create corresponding filters in the DNS sublayer.
    //

    FWPM_FILTER_CONDITION0 dnscond[2] = { 0, 0 };

    dnscond[0].fieldKey = FWPM_CONDITION_IP_REMOTE_PORT;
    dnscond[0].matchType = FWP_MATCH_EQUAL;
    dnscond[0].conditionValue.type = FWP_UINT16;
    dnscond[0].conditionValue.uint16 = DNS_SERVER_PORT;

    filter.filterCondition = dnscond;

    if (TunnelIpv6 != NULL) {
        dnscond[1] = cond;
        filter.numFilterConditions = 2;
    } else {
        filter.numFilterConditions = 1;
    }

    //
    // Ipv6 outbound DNS.
    //

    filter.filterKey = ST_FW_FILTER_PERMIT_SPLIT_APPS_IPV6_DNS_CONN_KEY;
    filter.layerKey = FWPM_LAYER_ALE_AUTH_CONNECT_V6;
    filter.subLayerKey = ST_FW_WINFW_DNS_SUBLAYER_KEY;
    filter.action.calloutKey = ST_FW_CALLOUT_PERMIT_SPLIT_APPS_IPV6_CONN_KEY;

    SUCCEED_OR_RETURN(FwpmFilterAdd0(WfpSession, &filter, NULL, NULL));

    //
    // Ipv6 inbound DNS.
    //

    filter.filterKey = ST_FW_FILTER_PERMIT_SPLIT_APPS_IPV6_DNS_RECV_KEY;
    filter.layerKey = FWPM_LAYER_ALE_AUTH_RECV_ACCEPT_V6;
    filter.action.calloutKey = ST_FW_CALLOUT_PERMIT_SPLIT_APPS_IPV6_RECV_KEY;

    return FwpmFilterAdd0(WfpSession, &filter, NULL, NULL);
}

NTSTATUS
RemoveFilterPermitNonTunnelIpv6Tx(HANDLE WfpSession) {
    SUCCEED_OR_RETURN(FwpmFilterDeleteByKey0(WfpSession, &ST_FW_FILTER_PERMIT_SPLIT_APPS_IPV6_CONN_KEY));

    SUCCEED_OR_RETURN(FwpmFilterDeleteByKey0(WfpSession, &ST_FW_FILTER_PERMIT_SPLIT_APPS_IPV6_RECV_KEY));

    SUCCEED_OR_RETURN(FwpmFilterDeleteByKey0(WfpSession, &ST_FW_FILTER_PERMIT_SPLIT_APPS_IPV6_DNS_CONN_KEY));

    return FwpmFilterDeleteByKey0(WfpSession, &ST_FW_FILTER_PERMIT_SPLIT_APPS_IPV6_DNS_RECV_KEY);
}

NTSTATUS
RegisterFilterBlockTunnelIpv4Tx(HANDLE WfpSession, const IN_ADDR* TunnelIp) {
    //
    // Create filters that match all tunnel IPv4 traffic.
    //
    // The linked callout will then block all existing and attempted connections
    // that can be associated with apps that are being split.
    //

    FWPM_FILTER0 filter = { 0 };

    auto const filterNameOutbound = L"Nym VPN Split Tunnel IPv4 Blocking Filter (Outbound)";
    auto const filterDescription = L"Blocks tunnel IPv4 traffic for apps being split";

    filter.filterKey = ST_FW_FILTER_BLOCK_ALL_SPLIT_APPS_TUNNEL_IPV4_CONN_KEY;
    filter.displayData.name = const_cast<wchar_t*>(filterNameOutbound);
    filter.displayData.description = const_cast<wchar_t*>(filterDescription);
    filter.flags = FWPM_FILTER_FLAG_CLEAR_ACTION_RIGHT | FWPM_FILTER_FLAG_HAS_PROVIDER_CONTEXT;
    filter.providerKey = const_cast<GUID*>(&ST_FW_PROVIDER_KEY);
    filter.layerKey = FWPM_LAYER_ALE_AUTH_CONNECT_V4;
    filter.subLayerKey = ST_FW_WINFW_BASELINE_SUBLAYER_KEY;
    filter.weight.type = FWP_UINT64;
    filter.weight.uint64 = const_cast<UINT64*>(&ST_MAX_FILTER_WEIGHT);
    filter.action.type = FWP_ACTION_CALLOUT_UNKNOWN;
    filter.action.calloutKey = ST_FW_CALLOUT_BLOCK_SPLIT_APPS_IPV4_CONN_KEY;
    filter.providerContextKey = ST_FW_PROVIDER_CONTEXT_KEY;

    FWPM_FILTER_CONDITION0 cond;

    cond.fieldKey = FWPM_CONDITION_IP_LOCAL_ADDRESS;
    cond.matchType = FWP_MATCH_EQUAL;
    cond.conditionValue.type = FWP_UINT32;
    cond.conditionValue.uint32 = RtlUlongByteSwap(TunnelIp->s_addr);

    filter.filterCondition = &cond;
    filter.numFilterConditions = 1;

    auto status = FwpmFilterAdd0(WfpSession, &filter, NULL, NULL);

    if (!NT_SUCCESS(status)) {
        return status;
    }

    auto const filterNameInbound = L"Nym VPN Split Tunnel IPv4 Blocking Filter (Inbound)";

    filter.filterKey = ST_FW_FILTER_BLOCK_ALL_SPLIT_APPS_TUNNEL_IPV4_RECV_KEY;
    filter.displayData.name = const_cast<wchar_t*>(filterNameInbound);
    filter.layerKey = FWPM_LAYER_ALE_AUTH_RECV_ACCEPT_V4;
    filter.action.calloutKey = ST_FW_CALLOUT_BLOCK_SPLIT_APPS_IPV4_RECV_KEY;

    return FwpmFilterAdd0(WfpSession, &filter, NULL, NULL);
}

NTSTATUS
RemoveFilterBlockTunnelIpv4Tx(HANDLE WfpSession) {
    auto status = FwpmFilterDeleteByKey0(WfpSession, &ST_FW_FILTER_BLOCK_ALL_SPLIT_APPS_TUNNEL_IPV4_CONN_KEY);

    if (!NT_SUCCESS(status)) {
        return status;
    }

    return FwpmFilterDeleteByKey0(WfpSession, &ST_FW_FILTER_BLOCK_ALL_SPLIT_APPS_TUNNEL_IPV4_RECV_KEY);
}

NTSTATUS
RegisterFilterBlockTunnelIpv6Tx(HANDLE WfpSession, const IN6_ADDR* TunnelIp) {
    //
    // Create filters that match all tunnel IPv6 traffic.
    //
    // The linked callout will then block all existing and attempted connections
    // that can be associated with apps that are being split.
    //

    FWPM_FILTER0 filter = { 0 };

    auto const filterNameOutbound = L"Nym VPN Split Tunnel IPv6 Blocking Filter (Outbound)";
    auto const filterDescription = L"Blocks tunnel IPv6 traffic for apps being split";

    filter.filterKey = ST_FW_FILTER_BLOCK_ALL_SPLIT_APPS_TUNNEL_IPV6_CONN_KEY;
    filter.displayData.name = const_cast<wchar_t*>(filterNameOutbound);
    filter.displayData.description = const_cast<wchar_t*>(filterDescription);
    filter.flags = FWPM_FILTER_FLAG_CLEAR_ACTION_RIGHT | FWPM_FILTER_FLAG_HAS_PROVIDER_CONTEXT;
    filter.providerKey = const_cast<GUID*>(&ST_FW_PROVIDER_KEY);
    filter.layerKey = FWPM_LAYER_ALE_AUTH_CONNECT_V6;
    filter.subLayerKey = ST_FW_WINFW_BASELINE_SUBLAYER_KEY;
    filter.weight.type = FWP_UINT64;
    filter.weight.uint64 = const_cast<UINT64*>(&ST_MAX_FILTER_WEIGHT);
    filter.action.type = FWP_ACTION_CALLOUT_UNKNOWN;
    filter.action.calloutKey = ST_FW_CALLOUT_BLOCK_SPLIT_APPS_IPV6_CONN_KEY;
    filter.providerContextKey = ST_FW_PROVIDER_CONTEXT_KEY;

    FWPM_FILTER_CONDITION0 cond;

    cond.fieldKey = FWPM_CONDITION_IP_LOCAL_ADDRESS;
    cond.matchType = FWP_MATCH_EQUAL;
    cond.conditionValue.type = FWP_BYTE_ARRAY16_TYPE;
    cond.conditionValue.byteArray16 = (FWP_BYTE_ARRAY16*)TunnelIp->u.Byte;

    filter.filterCondition = &cond;
    filter.numFilterConditions = 1;

    auto status = FwpmFilterAdd0(WfpSession, &filter, NULL, NULL);

    if (!NT_SUCCESS(status)) {
        return status;
    }

    auto const filterNameInbound = L"Nym VPN Split Tunnel IPv6 Blocking Filter (Inbound)";

    filter.filterKey = ST_FW_FILTER_BLOCK_ALL_SPLIT_APPS_TUNNEL_IPV6_RECV_KEY;
    filter.displayData.name = const_cast<wchar_t*>(filterNameInbound);
    filter.layerKey = FWPM_LAYER_ALE_AUTH_RECV_ACCEPT_V6;
    filter.action.calloutKey = ST_FW_CALLOUT_BLOCK_SPLIT_APPS_IPV6_RECV_KEY;

    return FwpmFilterAdd0(WfpSession, &filter, NULL, NULL);
}

NTSTATUS
RemoveFilterBlockTunnelIpv6Tx(HANDLE WfpSession) {
    auto status = FwpmFilterDeleteByKey0(WfpSession, &ST_FW_FILTER_BLOCK_ALL_SPLIT_APPS_TUNNEL_IPV6_CONN_KEY);

    if (!NT_SUCCESS(status)) {
        return status;
    }

    return FwpmFilterDeleteByKey0(WfpSession, &ST_FW_FILTER_BLOCK_ALL_SPLIT_APPS_TUNNEL_IPV6_RECV_KEY);
}

} // namespace firewall
