#include "stdafx.h"
#include "mullvadguids.h"
#include <algorithm>
#include <iterator>

//static
MullvadGuids::DetailedIdentityRegistry MullvadGuids::DeprecatedIdentities()
{
	//
	// Collect GUIDs here that were in use in previous versions of the app.
	//
	// Otherwise upgrades will fail because the upgraded daemon will fail to
	// remove sublayers etc because they contain filters that the updated code
	// doesn't know about.
	//

	std::multimap<WfpObjectType, GUID> registry;

	return registry;
}

//static
MullvadGuids::IdentityRegistry MullvadGuids::Registry(IdentityQualifier qualifier)
{
	const auto detailedRegistry = DetailedRegistry(qualifier);
	using ValueType = decltype(detailedRegistry)::const_reference;

	std::unordered_set<GUID> registry;

	std::transform(detailedRegistry.begin(), detailedRegistry.end(), std::inserter(registry, registry.end()), [](ValueType value)
	{
		return value.second;
	});

	return registry;
}

//static
MullvadGuids::DetailedIdentityRegistry MullvadGuids::DetailedRegistry(IdentityQualifier qualifier)
{
	std::multimap<WfpObjectType, GUID> registry;

	if (IdentityQualifier::IncludeDeprecated == (qualifier & IdentityQualifier::IncludeDeprecated))
	{
		registry = DeprecatedIdentities();
	}

	registry.insert(std::make_pair(WfpObjectType::Provider, Provider()));
	registry.insert(std::make_pair(WfpObjectType::Sublayer, SublayerBaseline()));
	registry.insert(std::make_pair(WfpObjectType::Sublayer, SublayerDns()));

	registry.insert(std::make_pair(WfpObjectType::Filter, Filter_Baseline_BlockAll_Outbound_Ipv4()));
	registry.insert(std::make_pair(WfpObjectType::Filter, Filter_Baseline_BlockAll_Inbound_Ipv4()));
	registry.insert(std::make_pair(WfpObjectType::Filter, Filter_Baseline_BlockAll_Outbound_Ipv6()));
	registry.insert(std::make_pair(WfpObjectType::Filter, Filter_Baseline_BlockAll_Inbound_Ipv6()));

	registry.insert(std::make_pair(WfpObjectType::Filter, Filter_Baseline_PermitLan_Outbound_Ipv4()));
	registry.insert(std::make_pair(WfpObjectType::Filter, Filter_Baseline_PermitLan_Outbound_Multicast_Ipv4()));
	registry.insert(std::make_pair(WfpObjectType::Filter, Filter_Baseline_PermitLan_Outbound_Ipv6()));
	registry.insert(std::make_pair(WfpObjectType::Filter, Filter_Baseline_PermitLan_Outbound_Multicast_Ipv6()));

	registry.insert(std::make_pair(WfpObjectType::Filter, Filter_Baseline_PermitLanService_Inbound_Ipv4()));
	registry.insert(std::make_pair(WfpObjectType::Filter, Filter_Baseline_PermitLanService_Inbound_Ipv6()));

	registry.insert(std::make_pair(WfpObjectType::Filter, Filter_Baseline_PermitLoopback_Outbound_Ipv4()));
	registry.insert(std::make_pair(WfpObjectType::Filter, Filter_Baseline_PermitLoopback_Inbound_Ipv4()));
	registry.insert(std::make_pair(WfpObjectType::Filter, Filter_Baseline_PermitLoopback_Outbound_Ipv6()));
	registry.insert(std::make_pair(WfpObjectType::Filter, Filter_Baseline_PermitLoopback_Inbound_Ipv6()));

	registry.insert(std::make_pair(WfpObjectType::Filter, Filter_Baseline_PermitDhcp_Outbound_Request_Ipv4()));
	registry.insert(std::make_pair(WfpObjectType::Filter, Filter_Baseline_PermitDhcp_Inbound_Response_Ipv4()));
	registry.insert(std::make_pair(WfpObjectType::Filter, Filter_Baseline_PermitDhcp_Outbound_Request_Ipv6()));
	registry.insert(std::make_pair(WfpObjectType::Filter, Filter_Baseline_PermitDhcp_Inbound_Response_Ipv6()));
	registry.insert(std::make_pair(WfpObjectType::Filter, Filter_Baseline_PermitDhcpServer_Inbound_Request_Ipv4()));
	registry.insert(std::make_pair(WfpObjectType::Filter, Filter_Baseline_PermitDhcpServer_Outbound_Response_Ipv4()));

	registry.insert(std::make_pair(WfpObjectType::Filter, Filter_Baseline_PermitVpnRelay_Ipv4_1()));
	registry.insert(std::make_pair(WfpObjectType::Filter, Filter_Baseline_PermitVpnRelay_Ipv4_2()));
	registry.insert(std::make_pair(WfpObjectType::Filter, Filter_Baseline_PermitVpnRelay_Ipv6_1()));
	registry.insert(std::make_pair(WfpObjectType::Filter, Filter_Baseline_PermitVpnRelay_Ipv6_2()));

	registry.insert(std::make_pair(WfpObjectType::Filter, Filter_Baseline_PermitEndpoint_Ipv4_1()));
	registry.insert(std::make_pair(WfpObjectType::Filter, Filter_Baseline_PermitEndpoint_Ipv4_2()));
	registry.insert(std::make_pair(WfpObjectType::Filter, Filter_Baseline_PermitEndpoint_Ipv4_3()));
	registry.insert(std::make_pair(WfpObjectType::Filter, Filter_Baseline_PermitEndpoint_Ipv4_4()));
	registry.insert(std::make_pair(WfpObjectType::Filter, Filter_Baseline_PermitEndpoint_Ipv4_5()));
	registry.insert(std::make_pair(WfpObjectType::Filter, Filter_Baseline_PermitEndpoint_Ipv4_6()));
	registry.insert(std::make_pair(WfpObjectType::Filter, Filter_Baseline_PermitEndpoint_Ipv4_7()));
	registry.insert(std::make_pair(WfpObjectType::Filter, Filter_Baseline_PermitEndpoint_Ipv4_8()));
	registry.insert(std::make_pair(WfpObjectType::Filter, Filter_Baseline_PermitEndpoint_Ipv4_9()));
	registry.insert(std::make_pair(WfpObjectType::Filter, Filter_Baseline_PermitEndpoint_Ipv4_10()));
	registry.insert(std::make_pair(WfpObjectType::Filter, Filter_Baseline_PermitEndpoint_Ipv4_11()));
	registry.insert(std::make_pair(WfpObjectType::Filter, Filter_Baseline_PermitEndpoint_Ipv4_12()));
	registry.insert(std::make_pair(WfpObjectType::Filter, Filter_Baseline_PermitEndpoint_Ipv4_13()));
	registry.insert(std::make_pair(WfpObjectType::Filter, Filter_Baseline_PermitEndpoint_Ipv4_14()));
	registry.insert(std::make_pair(WfpObjectType::Filter, Filter_Baseline_PermitEndpoint_Ipv4_15()));
	registry.insert(std::make_pair(WfpObjectType::Filter, Filter_Baseline_PermitEndpoint_Ipv4_16()));
	registry.insert(std::make_pair(WfpObjectType::Filter, Filter_Baseline_PermitEndpoint_Ipv4_17()));
	registry.insert(std::make_pair(WfpObjectType::Filter, Filter_Baseline_PermitEndpoint_Ipv4_18()));
	registry.insert(std::make_pair(WfpObjectType::Filter, Filter_Baseline_PermitEndpoint_Ipv4_19()));
	registry.insert(std::make_pair(WfpObjectType::Filter, Filter_Baseline_PermitEndpoint_Ipv4_20()));
	registry.insert(std::make_pair(WfpObjectType::Filter, Filter_Baseline_PermitEndpoint_Ipv4_21()));
	registry.insert(std::make_pair(WfpObjectType::Filter, Filter_Baseline_PermitEndpoint_Ipv4_22()));
	registry.insert(std::make_pair(WfpObjectType::Filter, Filter_Baseline_PermitEndpoint_Ipv4_23()));
	registry.insert(std::make_pair(WfpObjectType::Filter, Filter_Baseline_PermitEndpoint_Ipv4_24()));

	registry.insert(std::make_pair(WfpObjectType::Filter, Filter_Baseline_PermitEndpoint_Ipv6_1()));
	registry.insert(std::make_pair(WfpObjectType::Filter, Filter_Baseline_PermitEndpoint_Ipv6_2()));
	registry.insert(std::make_pair(WfpObjectType::Filter, Filter_Baseline_PermitEndpoint_Ipv6_3()));
	registry.insert(std::make_pair(WfpObjectType::Filter, Filter_Baseline_PermitEndpoint_Ipv6_4()));
	registry.insert(std::make_pair(WfpObjectType::Filter, Filter_Baseline_PermitEndpoint_Ipv6_5()));
	registry.insert(std::make_pair(WfpObjectType::Filter, Filter_Baseline_PermitEndpoint_Ipv6_6()));
	registry.insert(std::make_pair(WfpObjectType::Filter, Filter_Baseline_PermitEndpoint_Ipv6_7()));
	registry.insert(std::make_pair(WfpObjectType::Filter, Filter_Baseline_PermitEndpoint_Ipv6_8()));
	registry.insert(std::make_pair(WfpObjectType::Filter, Filter_Baseline_PermitEndpoint_Ipv6_9()));
	registry.insert(std::make_pair(WfpObjectType::Filter, Filter_Baseline_PermitEndpoint_Ipv6_10()));
	registry.insert(std::make_pair(WfpObjectType::Filter, Filter_Baseline_PermitEndpoint_Ipv6_11()));
	registry.insert(std::make_pair(WfpObjectType::Filter, Filter_Baseline_PermitEndpoint_Ipv6_12()));
	registry.insert(std::make_pair(WfpObjectType::Filter, Filter_Baseline_PermitEndpoint_Ipv6_13()));
	registry.insert(std::make_pair(WfpObjectType::Filter, Filter_Baseline_PermitEndpoint_Ipv6_14()));
	registry.insert(std::make_pair(WfpObjectType::Filter, Filter_Baseline_PermitEndpoint_Ipv6_15()));
	registry.insert(std::make_pair(WfpObjectType::Filter, Filter_Baseline_PermitEndpoint_Ipv6_16()));
	registry.insert(std::make_pair(WfpObjectType::Filter, Filter_Baseline_PermitEndpoint_Ipv6_17()));
	registry.insert(std::make_pair(WfpObjectType::Filter, Filter_Baseline_PermitEndpoint_Ipv6_18()));
	registry.insert(std::make_pair(WfpObjectType::Filter, Filter_Baseline_PermitEndpoint_Ipv6_19()));
	registry.insert(std::make_pair(WfpObjectType::Filter, Filter_Baseline_PermitEndpoint_Ipv6_20()));
	registry.insert(std::make_pair(WfpObjectType::Filter, Filter_Baseline_PermitEndpoint_Ipv6_21()));
	registry.insert(std::make_pair(WfpObjectType::Filter, Filter_Baseline_PermitEndpoint_Ipv6_22()));
	registry.insert(std::make_pair(WfpObjectType::Filter, Filter_Baseline_PermitEndpoint_Ipv6_23()));
	registry.insert(std::make_pair(WfpObjectType::Filter, Filter_Baseline_PermitEndpoint_Ipv6_24()));

	registry.insert(std::make_pair(WfpObjectType::Filter, Filter_Baseline_PermitVpnTunnel_Entry_Outbound_Ipv4_1()));
	registry.insert(std::make_pair(WfpObjectType::Filter, Filter_Baseline_PermitVpnTunnel_Entry_Outbound_Ipv6_1()));
	registry.insert(std::make_pair(WfpObjectType::Filter, Filter_Baseline_PermitVpnTunnel_Entry_Outbound_Ipv4_2()));
	registry.insert(std::make_pair(WfpObjectType::Filter, Filter_Baseline_PermitVpnTunnel_Entry_Outbound_Ipv6_2()));

	registry.insert(std::make_pair(WfpObjectType::Filter, Filter_Baseline_PermitVpnTunnel_Exit_Outbound_Ipv4_1()));
	registry.insert(std::make_pair(WfpObjectType::Filter, Filter_Baseline_PermitVpnTunnel_Exit_Outbound_Ipv6_1()));
	registry.insert(std::make_pair(WfpObjectType::Filter, Filter_Baseline_PermitVpnTunnel_Exit_Outbound_Ipv4_2()));
	registry.insert(std::make_pair(WfpObjectType::Filter, Filter_Baseline_PermitVpnTunnel_Exit_Outbound_Ipv6_2()));

	registry.insert(std::make_pair(WfpObjectType::Filter, Filter_Baseline_PermitVpnTunnelService_Entry_Ipv4_1()));
	registry.insert(std::make_pair(WfpObjectType::Filter, Filter_Baseline_PermitVpnTunnelService_Entry_Ipv6_1()));
	registry.insert(std::make_pair(WfpObjectType::Filter, Filter_Baseline_PermitVpnTunnelService_Entry_Ipv4_2()));
	registry.insert(std::make_pair(WfpObjectType::Filter, Filter_Baseline_PermitVpnTunnelService_Entry_Ipv6_2()));

	registry.insert(std::make_pair(WfpObjectType::Filter, Filter_Baseline_PermitVpnTunnelService_Exit_Ipv4_1()));
	registry.insert(std::make_pair(WfpObjectType::Filter, Filter_Baseline_PermitVpnTunnelService_Exit_Ipv6_1()));
	registry.insert(std::make_pair(WfpObjectType::Filter, Filter_Baseline_PermitVpnTunnelService_Exit_Ipv4_2()));
	registry.insert(std::make_pair(WfpObjectType::Filter, Filter_Baseline_PermitVpnTunnelService_Exit_Ipv6_2()));

	registry.insert(std::make_pair(WfpObjectType::Filter, Filter_Baseline_PermitNdp_Outbound_Router_Solicitation()));
	registry.insert(std::make_pair(WfpObjectType::Filter, Filter_Baseline_PermitNdp_Inbound_Router_Advertisement()));
	registry.insert(std::make_pair(WfpObjectType::Filter, Filter_Baseline_PermitNdp_Outbound_Neighbor_Solicitation()));
	registry.insert(std::make_pair(WfpObjectType::Filter, Filter_Baseline_PermitNdp_Inbound_Neighbor_Solicitation()));
	registry.insert(std::make_pair(WfpObjectType::Filter, Filter_Baseline_PermitNdp_Outbound_Neighbor_Advertisement()));
	registry.insert(std::make_pair(WfpObjectType::Filter, Filter_Baseline_PermitNdp_Inbound_Neighbor_Advertisement()));
	registry.insert(std::make_pair(WfpObjectType::Filter, Filter_Baseline_PermitNdp_Inbound_Redirect()));

	registry.insert(std::make_pair(WfpObjectType::Filter, Filter_Baseline_PermitDns_Outbound_Ipv4()));
	registry.insert(std::make_pair(WfpObjectType::Filter, Filter_Baseline_PermitDns_Outbound_Ipv6()));

	registry.insert(std::make_pair(WfpObjectType::Filter, Filter_Dns_BlockAll_Outbound_Ipv4()));
	registry.insert(std::make_pair(WfpObjectType::Filter, Filter_Dns_BlockAll_Outbound_Ipv6()));

	registry.insert(std::make_pair(WfpObjectType::Filter, Filter_Dns_PermitLoopback_Outbound_Ipv4()));
	registry.insert(std::make_pair(WfpObjectType::Filter, Filter_Dns_PermitLoopback_Outbound_Ipv6()));

	registry.insert(std::make_pair(WfpObjectType::Filter, Filter_Dns_PermitNonTunnel_Outbound_Ipv4()));
	registry.insert(std::make_pair(WfpObjectType::Filter, Filter_Dns_PermitNonTunnel_Outbound_Ipv6()));

	registry.insert(std::make_pair(WfpObjectType::Filter, Filter_Dns_PermitTunnel_Outbound_Ipv4()));
	registry.insert(std::make_pair(WfpObjectType::Filter, Filter_Dns_PermitTunnel_Outbound_Ipv6()));

	if (IdentityQualifier::IncludePersistent == (qualifier & IdentityQualifier::IncludePersistent))
	{
		registry.insert(std::make_pair(WfpObjectType::Provider, ProviderPersistent()));
		registry.insert(std::make_pair(WfpObjectType::Sublayer, SublayerPersistent()));

		registry.insert(std::make_pair(WfpObjectType::Filter, Filter_Boottime_BlockAll_Inbound_Ipv4()));
		registry.insert(std::make_pair(WfpObjectType::Filter, Filter_Boottime_BlockAll_Outbound_Ipv4()));
		registry.insert(std::make_pair(WfpObjectType::Filter, Filter_Boottime_BlockAll_Inbound_Ipv6()));
		registry.insert(std::make_pair(WfpObjectType::Filter, Filter_Boottime_BlockAll_Outbound_Ipv6()));

		registry.insert(std::make_pair(WfpObjectType::Filter, Filter_Persistent_BlockAll_Inbound_Ipv4()));
		registry.insert(std::make_pair(WfpObjectType::Filter, Filter_Persistent_BlockAll_Outbound_Ipv4()));
		registry.insert(std::make_pair(WfpObjectType::Filter, Filter_Persistent_BlockAll_Inbound_Ipv6()));
		registry.insert(std::make_pair(WfpObjectType::Filter, Filter_Persistent_BlockAll_Outbound_Ipv6()));
	}

	return registry;
}

//static
const GUID &MullvadGuids::Provider()
{
	// {C736D993-9447-4982-8DD1-EEE10461EF3C}
	static const GUID g = { 0xc736d993,0x9447,0x4982,{0x8d,0xd1,0xee,0xe1,0x04,0x61,0xef,0x3c} };

	return g;
}

//static
const GUID &MullvadGuids::ProviderPersistent()
{
	// {26646A94-C70A-47A3-AC66-114BACFA556A}
	static const GUID g = { 0x26646a94,0xc70a,0x47a3,{0xac,0x66,0x11,0x4b,0xac,0xfa,0x55,0x6a} };

	return g;
}

//static
const GUID &MullvadGuids::SublayerBaseline()
{
	// {25A0D4A1-5FD3-4D32-9252-34A7B47A7D2E}
	static const GUID g = { 0x25a0d4a1,0x5fd3,0x4d32,{0x92,0x52,0x34,0xa7,0xb4,0x7a,0x7d,0x2e} };

	return g;
}

//static
const GUID &MullvadGuids::SublayerDns()
{
	// {3FDEC7AA-9CF9-4F8B-980F-8AADE8BA0DC6}
	static const GUID g = { 0x3fdec7aa,0x9cf9,0x4f8b,{0x98,0x0f,0x8a,0xad,0xe8,0xba,0x0d,0xc6} };

	return g;
}

//static
const GUID &MullvadGuids::SublayerPersistent()
{
	// {7F81AB43-6F94-4772-B3D2-17DB757BBE3B}
	static const GUID g = { 0x7f81ab43,0x6f94,0x4772,{0xb3,0xd2,0x17,0xdb,0x75,0x7b,0xbe,0x3b} };

	return g;
}

//static
const GUID &MullvadGuids::Filter_Boottime_BlockAll_Outbound_Ipv4()
{
	// {C862565B-EE75-4065-9A92-5D72A6569B28}
	static const GUID g = { 0xc862565b,0xee75,0x4065,{0x9a,0x92,0x5d,0x72,0xa6,0x56,0x9b,0x28} };

	return g;
}

//static
const GUID &MullvadGuids::Filter_Boottime_BlockAll_Inbound_Ipv4()
{
	// {71881E3A-D74A-45AD-B983-FB5BFEDECF62}
	static const GUID g = { 0x71881e3a,0xd74a,0x45ad,{0xb9,0x83,0xfb,0x5b,0xfe,0xde,0xcf,0x62} };

	return g;
}

//static
const GUID &MullvadGuids::Filter_Boottime_BlockAll_Outbound_Ipv6()
{
	// {107B140F-0195-473A-B308-611B77D0600B}
	static const GUID g = { 0x107b140f,0x0195,0x473a,{0xb3,0x08,0x61,0x1b,0x77,0xd0,0x60,0x0b} };

	return g;
}

//static
const GUID &MullvadGuids::Filter_Boottime_BlockAll_Inbound_Ipv6()
{
	// {FA8DB319-85BC-458D-98FF-F9071F0D69BB}
	static const GUID g = { 0xfa8db319,0x85bc,0x458d,{0x98,0xff,0xf9,0x07,0x1f,0x0d,0x69,0xbb} };

	return g;
}

//static
const GUID &MullvadGuids::Filter_Persistent_BlockAll_Outbound_Ipv4()
{
	// {E94F85E6-0C56-4EE5-BC63-EEE0AB759A37}
	static const GUID g = { 0xe94f85e6,0x0c56,0x4ee5,{0xbc,0x63,0xee,0xe0,0xab,0x75,0x9a,0x37} };

	return g;
}

//static
const GUID &MullvadGuids::Filter_Persistent_BlockAll_Inbound_Ipv4()
{
	// {F5C393B1-BFB6-4845-9618-968984A6A389}
	static const GUID g = { 0xf5c393b1,0xbfb6,0x4845,{0x96,0x18,0x96,0x89,0x84,0xa6,0xa3,0x89} };

	return g;
}

//static
const GUID &MullvadGuids::Filter_Persistent_BlockAll_Outbound_Ipv6()
{
	// {3B8F4102-5CDD-41F4-9E7E-FE776BF42260}
	static const GUID g = { 0x3b8f4102,0x5cdd,0x41f4,{0x9e,0x7e,0xfe,0x77,0x6b,0xf4,0x22,0x60} };

	return g;
}

//static
const GUID &MullvadGuids::Filter_Persistent_BlockAll_Inbound_Ipv6()
{
	// {F09E4128-B482-4853-9235-FDF43C1ED314}
	static const GUID g = { 0xf09e4128,0xb482,0x4853,{0x92,0x35,0xfd,0xf4,0x3c,0x1e,0xd3,0x14} };

	return g;
}

//static
const GUID &MullvadGuids::Filter_Baseline_BlockAll_Outbound_Ipv4()
{
	// {57AE8F49-2583-4D3E-8AB7-66A9BCBC8866}
	static const GUID g = { 0x57ae8f49,0x2583,0x4d3e,{0x8a,0xb7,0x66,0xa9,0xbc,0xbc,0x88,0x66} };

	return g;
}

//static
const GUID &MullvadGuids::Filter_Baseline_BlockAll_Inbound_Ipv4()
{
	// {B0E82D34-2534-4B58-B6FF-7E7AE7EAD7A4}
	static const GUID g = { 0xb0e82d34,0x2534,0x4b58,{0xb6,0xff,0x7e,0x7a,0xe7,0xea,0xd7,0xa4} };

	return g;
}

//static
const GUID &MullvadGuids::Filter_Baseline_BlockAll_Outbound_Ipv6()
{
	// {02FE62FF-68A1-453B-AE36-97EAF87C15DC}
	static const GUID g = { 0x02fe62ff,0x68a1,0x453b,{0xae,0x36,0x97,0xea,0xf8,0x7c,0x15,0xdc} };

	return g;
}

//static
const GUID &MullvadGuids::Filter_Baseline_BlockAll_Inbound_Ipv6()
{
	// {C5275E0A-21B7-4F80-8866-FFEA23AEF600}
	static const GUID g = { 0xc5275e0a,0x21b7,0x4f80,{0x88,0x66,0xff,0xea,0x23,0xae,0xf6,0x00} };

	return g;
}


//static
const GUID &MullvadGuids::Filter_Baseline_PermitLan_Outbound_Ipv4()
{
	// {743DC4E9-052F-4215-AA4A-F9417F3D31EC}
	static const GUID g = { 0x743dc4e9,0x052f,0x4215,{0xaa,0x4a,0xf9,0x41,0x7f,0x3d,0x31,0xec} };

	return g;
}

//static
const GUID &MullvadGuids::Filter_Baseline_PermitLan_Outbound_Multicast_Ipv4()
{
	// {B0131301-4967-4EE0-ADF3-36F2E13118A7}
	static const GUID g = { 0xb0131301,0x4967,0x4ee0,{0xad,0xf3,0x36,0xf2,0xe1,0x31,0x18,0xa7} };

	return g;
}

//static
const GUID &MullvadGuids::Filter_Baseline_PermitLan_Outbound_Ipv6()
{
	// {031FA5DE-BD10-440E-B387-41F30A52FC5D}
	static const GUID g = { 0x031fa5de,0xbd10,0x440e,{0xb3,0x87,0x41,0xf3,0x0a,0x52,0xfc,0x5d} };

	return g;
}

//static
const GUID &MullvadGuids::Filter_Baseline_PermitLan_Outbound_Multicast_Ipv6()
{
	// {70D3E5DF-9D56-4242-8A69-75304D58028D}
	static const GUID g = { 0x70d3e5df,0x9d56,0x4242,{0x8a,0x69,0x75,0x30,0x4d,0x58,0x02,0x8d} };

	return g;
}

//static
const GUID &MullvadGuids::Filter_Baseline_PermitLanService_Inbound_Ipv4()
{
	// {8DC88A9F-A6D7-4C10-8143-F38FF4A463D6}
	static const GUID g = { 0x8dc88a9f,0xa6d7,0x4c10,{0x81,0x43,0xf3,0x8f,0xf4,0xa4,0x63,0xd6} };

	return g;
}

//static
const GUID &MullvadGuids::Filter_Baseline_PermitLanService_Inbound_Ipv6()
{
	// {B3F7BF6D-0250-4A80-A9B8-2126E3169626}
	static const GUID g = { 0xb3f7bf6d,0x0250,0x4a80,{0xa9,0xb8,0x21,0x26,0xe3,0x16,0x96,0x26} };

	return g;
}

//static
const GUID &MullvadGuids::Filter_Baseline_PermitLoopback_Outbound_Ipv4()
{
	// {4C62148B-D5A7-4981-AB93-ADEA54D7D191}
	static const GUID g = { 0x4c62148b,0xd5a7,0x4981,{0xab,0x93,0xad,0xea,0x54,0xd7,0xd1,0x91} };

	return g;
}

//static
const GUID &MullvadGuids::Filter_Baseline_PermitLoopback_Inbound_Ipv4()
{
	// {DF187B6E-CE45-4653-841A-F80401C9DB00}
	static const GUID g = { 0xdf187b6e,0xce45,0x4653,{0x84,0x1a,0xf8,0x04,0x01,0xc9,0xdb,0x00} };

	return g;
}

//static
const GUID &MullvadGuids::Filter_Baseline_PermitLoopback_Outbound_Ipv6()
{
	// {D9199D45-BC8D-4935-9A36-7FC52AFB3CF8}
	static const GUID g = { 0xd9199d45,0xbc8d,0x4935,{0x9a,0x36,0x7f,0xc5,0x2a,0xfb,0x3c,0xf8} };

	return g;
}

//static
const GUID &MullvadGuids::Filter_Baseline_PermitLoopback_Inbound_Ipv6()
{
	// {BBEC26D8-7F2E-4141-BB51-BBB4C9FA7292}
	static const GUID g = { 0xbbec26d8,0x7f2e,0x4141,{0xbb,0x51,0xbb,0xb4,0xc9,0xfa,0x72,0x92} };

	return g;
}

//static
const GUID &MullvadGuids::Filter_Baseline_PermitDhcp_Outbound_Request_Ipv4()
{
	// {4FC8EEC2-C8CB-4B6F-A9EC-4B0255E0F676}
	static const GUID g = { 0x4fc8eec2,0xc8cb,0x4b6f,{0xa9,0xec,0x4b,0x02,0x55,0xe0,0xf6,0x76} };

	return g;
}

//static
const GUID &MullvadGuids::Filter_Baseline_PermitDhcp_Inbound_Response_Ipv4()
{
	// {577C6D56-EF10-4ADA-8AB6-BB22C7BADF42}
	static const GUID g = { 0x577c6d56,0xef10,0x4ada,{0x8a,0xb6,0xbb,0x22,0xc7,0xba,0xdf,0x42} };

	return g;
}

//static
const GUID &MullvadGuids::Filter_Baseline_PermitDhcp_Outbound_Request_Ipv6()
{
	// {5D91A7C0-A9A9-43C7-A95F-B8733C14F8D7}
	static const GUID g = { 0x5d91a7c0,0xa9a9,0x43c7,{0xa9,0x5f,0xb8,0x73,0x3c,0x14,0xf8,0xd7} };

	return g;
}

//static
const GUID &MullvadGuids::Filter_Baseline_PermitDhcp_Inbound_Response_Ipv6()
{
	// {5CFE4773-A8DC-4770-A0AE-B58478511D8C}
	static const GUID g = { 0x5cfe4773,0xa8dc,0x4770,{0xa0,0xae,0xb5,0x84,0x78,0x51,0x1d,0x8c} };

	return g;
}

//static
const GUID &MullvadGuids::Filter_Baseline_PermitDhcpServer_Inbound_Request_Ipv4()
{
	// {C7FAC6E7-E33E-48CC-96CA-7684E1B5F134}
	static const GUID g = { 0xc7fac6e7,0xe33e,0x48cc,{0x96,0xca,0x76,0x84,0xe1,0xb5,0xf1,0x34} };

	return g;
}

//static
const GUID &MullvadGuids::Filter_Baseline_PermitDhcpServer_Outbound_Response_Ipv4()
{
	// {84FA25B6-9F4F-416D-BCD5-7CB5932CD088}
	static const GUID g = { 0x84fa25b6,0x9f4f,0x416d,{0xbc,0xd5,0x7c,0xb5,0x93,0x2c,0xd0,0x88} };

	return g;
}

//static
const GUID &MullvadGuids::Filter_Baseline_PermitVpnRelay_Ipv4_1()
{
	// {93E92E50-FA3F-45D9-B576-8AB1233269A3}
	static const GUID g = { 0x93e92e50,0xfa3f,0x45d9,{0xb5,0x76,0x8a,0xb1,0x23,0x32,0x69,0xa3} };

	return g;
}

//static
const GUID &MullvadGuids::Filter_Baseline_PermitVpnRelay_Ipv4_2()
{
	// {1F484D78-F9B8-43C1-9930-883EF830431F}
	static const GUID g = { 0x1f484d78, 0xf9b8, 0x43c1, { 0x99, 0x30, 0x88, 0x3e, 0xf8, 0x30, 0x43, 0x1f } };

	return g;
}

//static
const GUID &MullvadGuids::Filter_Baseline_PermitVpnRelay_Ipv6_1()
{
	// {2E0D95D2-530E-4D35-9BA8-50458B971B46}
	static const GUID g = { 0x2e0d95d2, 0x530e, 0x4d35, { 0x9b, 0xa8, 0x50, 0x45, 0x8b, 0x97, 0x1b, 0x46 } };


	return g;
}

//static
const GUID &MullvadGuids::Filter_Baseline_PermitVpnRelay_Ipv6_2()
{
	// {A9893597-4FCA-49BC-99A1-ED3FC44DEA82}
	static const GUID g = { 0xa9893597, 0x4fca, 0x49bc, { 0x99, 0xa1, 0xed, 0x3f, 0xc4, 0x4d, 0xea, 0x82 } };


	return g;
}

//static
const GUID &MullvadGuids::Filter_Baseline_PermitEndpoint_Ipv4_1()
{
	// {AF5716AA-D4E4-4E3E-9E85-E53AB4479338}
	static const GUID g = { 0xaf5716aa,0xd4e4,0x4e3e,{0x9e,0x85,0xe5,0x3a,0xb4,0x47,0x93,0x38} };

	return g;
}

//static
const GUID &MullvadGuids::Filter_Baseline_PermitEndpoint_Ipv4_2()
{
	// {1F1D87EC-6022-48C9-BDAA-224C428E30C0}
	static const GUID g = { 0x1f1d87ec,0x6022,0x48c9,{0xbd,0xaa,0x22,0x4c,0x42,0x8e,0x30,0xc0} };

	return g;
}

//static
const GUID &MullvadGuids::Filter_Baseline_PermitEndpoint_Ipv4_3()
{
	// {CFDA4531-279D-4F4F-989C-93FB7C1C7AED}
	static const GUID g = { 0xcfda4531,0x279d,0x4f4f,{0x98,0x9c,0x93,0xfb,0x7c,0x1c,0x7a,0xed} };

	return g;
}

//static
const GUID &MullvadGuids::Filter_Baseline_PermitEndpoint_Ipv4_4()
{
	// {7CB2CBA7-AF0A-43C8-B86E-86405FBC6352}
	static const GUID g = { 0x7cb2cba7,0xaf0a,0x43c8,{0xb8,0x6e,0x86,0x40,0x5f,0xbc,0x63,0x52} };

	return g;
}

//static
const GUID& MullvadGuids::Filter_Baseline_PermitEndpoint_Ipv4_5() {
	// {BD23286A-F0F0-41C1-A46C-935F2C8875EF}
	static const GUID g = { 0xbd23286a,0xf0f0,0x41c1,{0xa4,0x6c,0x93,0x5f,0x2c,0x88,0x75,0xef} };

	return g;
}

//static
const GUID& MullvadGuids::Filter_Baseline_PermitEndpoint_Ipv4_6() {
	// {C6D38A13-7886-4D90-835B-DFF312F46BCB}
	static const GUID g = { 0xc6d38a13,0x7886,0x4d90,{0x83,0x5b,0xdf,0xf3,0x12,0xf4,0x6b,0xcb} };

	return g;
}

//static
const GUID& MullvadGuids::Filter_Baseline_PermitEndpoint_Ipv4_7() {
	// {C7905ABB-4C9C-40B6-A9F5-8A5951063668}
	static const GUID g = { 0xc7905abb,0x4c9c,0x40b6,{0xa9,0xf5,0x8a,0x59,0x51,0x06,0x36,0x68} };

	return g;
}

//static
const GUID& MullvadGuids::Filter_Baseline_PermitEndpoint_Ipv4_8() {
	// {5706FDD0-602A-4C91-BD57-A2D323DC7E7A}
	static const GUID g = { 0x5706fdd0,0x602a,0x4c91,{0xbd,0x57,0xa2,0xd3,0x23,0xdc,0x7e,0x7a} };

	return g;
}

//static
const GUID& MullvadGuids::Filter_Baseline_PermitEndpoint_Ipv4_9() {
	// {A6AACF88-3623-4D34-9AA9-028F8D63CCCD}
	static const GUID g = { 0xa6aacf88,0x3623,0x4d34,{0x9a,0xa9,0x02,0x8f,0x8d,0x63,0xcc,0xcd} };

	return g;
}

//static
const GUID& MullvadGuids::Filter_Baseline_PermitEndpoint_Ipv4_10() {
	// {6DAA753E-22D5-4F61-B0DF-895D30319D1F}
	static const GUID g = { 0x6daa753e,0x22d5,0x4f61,{0xb0,0xdf,0x89,0x5d,0x30,0x31,0x9d,0x1f} };

	return g;
}

//static
const GUID& MullvadGuids::Filter_Baseline_PermitEndpoint_Ipv4_11() {
	// {CCE1BBF0-36C2-4D8E-A1D0-BC61717C123E}
	static const GUID g = { 0xcce1bbf0,0x36c2,0x4d8e,{0xa1,0xd0,0xbc,0x61,0x71,0x7c,0x12,0x3e} };

	return g;
}

//static
const GUID& MullvadGuids::Filter_Baseline_PermitEndpoint_Ipv4_12() {
	// {DE9D2B08-A6FD-435F-AA6F-95C9793EA9F7}
	static const GUID g = { 0xde9d2b08,0xa6fd,0x435f,{0xaa,0x6f,0x95,0xc9,0x79,0x3e,0xa9,0xf7} };

	return g;
}

//static
const GUID& MullvadGuids::Filter_Baseline_PermitEndpoint_Ipv4_13() {
	// {B85C4D1E-4C39-49E1-9F5F-9C2C6D4A1B72}
	static const GUID g = { 0xb85c4d1e,0x4c39,0x49e1,{0x9f,0x5f,0x9c,0x2c,0x6d,0x4a,0x1b,0x72} };

	return g;
}

//static
const GUID& MullvadGuids::Filter_Baseline_PermitEndpoint_Ipv4_14() {
	// {E4D22A7B-7F0A-48D5-9C9E-4F1A2B3C5D6E}
	static const GUID g = { 0xe4d22a7b,0x7f0a,0x48d5,{0x9c,0x9e,0x4f,0x1a,0x2b,0x3c,0x5d,0x6e} };

	return g;
}

//static
const GUID& MullvadGuids::Filter_Baseline_PermitEndpoint_Ipv4_15() {
	// {4C9B1E2F-3A58-4F8C-AC1B-1DE2F3A4B5C6}
	static const GUID g = { 0x4c9b1e2f,0x3a58,0x4f8c,{0xac,0x1b,0x1d,0xe2,0xf3,0xa4,0xb5,0xc6} };

	return g;
}

//static
const GUID& MullvadGuids::Filter_Baseline_PermitEndpoint_Ipv4_16() {
	// {8A7E2C4D-6B31-4D2A-91E3-7F6A5B4C3D2E}
	static const GUID g = { 0x8a7e2c4d,0x6b31,0x4d2a,{0x91,0xe3,0x7f,0x6a,0x5b,0x4c,0x3d,0x2e} };

	return g;
}

//static
const GUID& MullvadGuids::Filter_Baseline_PermitEndpoint_Ipv4_17() {
	// {2F1C7B9A-5D84-4B0E-AF22-0A1B2C3D4E5F}
	static const GUID g = { 0x2f1c7b9a,0x5d84,0x4b0e,{0xaf,0x22,0x0a,0x1b,0x2c,0x3d,0x4e,0x5f} };

	return g;
}

//static
const GUID& MullvadGuids::Filter_Baseline_PermitEndpoint_Ipv4_18() {
	// {D3E5A9C1-1F2B-4A6D-8579-AB12CD34EF56}
	static const GUID g = { 0xd3e5a9c1,0x1f2b,0x4a6d,{0x85,0x79,0xab,0x12,0xcd,0x34,0xef,0x56} };

	return g;
}

//static
const GUID& MullvadGuids::Filter_Baseline_PermitEndpoint_Ipv4_19() {
	// {0E9C1A2B-3D4F-4B6A-8C7D-1E2F3A4B5C6D}
	static const GUID g = { 0x0e9c1a2b,0x3d4f,0x4b6a,{0x8c,0x7d,0x1e,0x2f,0x3a,0x4b,0x5c,0x6d} };

	return g;
}

//static
const GUID& MullvadGuids::Filter_Baseline_PermitEndpoint_Ipv4_20() {
	// {9A1B2C3D-4E5F-4768-90AB-CDEF12345678}
	static const GUID g = { 0x9a1b2c3d,0x4e5f,0x4768,{0x90,0xab,0xcd,0xef,0x12,0x34,0x56,0x78} };

	return g;
}

//static
const GUID& MullvadGuids::Filter_Baseline_PermitEndpoint_Ipv4_21() {
	// {6B4C3D2E-1F0A-4E9D-B2C3-4D5E6F708192}
	static const GUID g = { 0x6b4c3d2e,0x1f0a,0x4e9d,{0xb2,0xc3,0x4d,0x5e,0x6f,0x70,0x81,0x92} };

	return g;
}

//static
const GUID& MullvadGuids::Filter_Baseline_PermitEndpoint_Ipv4_22() {
	// {A4B5C6D7-E8F9-4A0B-8C9D-0E1F2A3B4C5D}
	static const GUID g = { 0xa4b5c6d7,0xe8f9,0x4a0b,{0x8c,0x9d,0x0e,0x1f,0x2a,0x3b,0x4c,0x5d} };

	return g;
}

//static
const GUID& MullvadGuids::Filter_Baseline_PermitEndpoint_Ipv4_23() {
	// {13F2A4C6-57B9-4D8E-9A0B-1C2D3E4F5061}
	static const GUID g = { 0x13f2a4c6,0x57b9,0x4d8e,{0x9a,0x0b,0x1c,0x2d,0x3e,0x4f,0x50,0x61} };

	return g;
}

//static
const GUID& MullvadGuids::Filter_Baseline_PermitEndpoint_Ipv4_24() {
	// {77C1B2A3-4D5E-6F70-8192-A3B4C5D6E7F8}
	static const GUID g = { 0x77c1b2a3,0x4d5e,0x6f70,{0x81,0x92,0xa3,0xb4,0xc5,0xd6,0xe7,0xf8} };

	return g;
}

//static
const GUID& MullvadGuids::Filter_Baseline_PermitEndpoint_Ipv6_1()
{
	// {C88C848F-2DF9-4908-944D-DE550CAD325E}
	static const GUID g = { 0xc88c848f,0x2df9,0x4908,{0x94,0x4d,0xde,0x55,0x0c,0xad,0x32,0x5e} };

	return g;
}

//static
const GUID& MullvadGuids::Filter_Baseline_PermitEndpoint_Ipv6_2()
{
	// {A8777D53-399B-418F-B24F-B03BAEABB68E}
	static const GUID g = { 0xa8777d53,0x399b,0x418f,{0xb2,0x4f,0xb0,0x3b,0xae,0xab,0xb6,0x8e} };

	return g;
}

//static
const GUID& MullvadGuids::Filter_Baseline_PermitEndpoint_Ipv6_3()
{
	// {ECE12F4D-EA16-4672-A128-43BE87A2D9C9}
	static const GUID g = { 0xece12f4d,0xea16,0x4672,{0xa1,0x28,0x43,0xbe,0x87,0xa2,0xd9,0xc9} };

	return g;
}


//static
const GUID& MullvadGuids::Filter_Baseline_PermitEndpoint_Ipv6_4()
{
	// {A6674EDA-3AA6-4937-B2DC-FAE0B1AE83BE}
	static const GUID g = { 0xa6674eda,0x3aa6,0x4937,{0xb2,0xdc,0xfa,0xe0,0xb1,0xae,0x83,0xbe} };

	return g;
}

//static
const GUID &MullvadGuids::Filter_Baseline_PermitEndpoint_Ipv6_5() {
	// {B0FC3204-8857-44E0-91F9-D3C0903BF6E8}
	static const GUID g = { 0xb0fc3204,0x8857,0x44e0,{0x91,0xf9,0xd3,0xc0,0x90,0x3b,0xf6,0xe8} };

	return g;
}

//static
const GUID &MullvadGuids::Filter_Baseline_PermitEndpoint_Ipv6_6() {
	// {92A0C2EB-F886-477F-8E91-2D503D185060}
	static const GUID g = { 0x92a0c2eb,0xf886,0x477f,{0x8e,0x91,0x2d,0x50,0x3d,0x18,0x50,0x60} };

	return g;
}

//static
const GUID &MullvadGuids::Filter_Baseline_PermitEndpoint_Ipv6_7() {
	// {55C9A462-39A8-42AF-A2A1-51D6E5AFA09C}
	static const GUID g = { 0x55c9a462,0x39a8,0x42af,{0xa2,0xa1,0x51,0xd6,0xe5,0xaf,0xa0,0x9c} };

	return g;
}

//static
const GUID &MullvadGuids::Filter_Baseline_PermitEndpoint_Ipv6_8() {
	// {5F3D9433-C688-4112-AA31-7E9760EA20B6}
	static const GUID g = { 0x5f3d9433,0xc688,0x4112,{0xaa,0x31,0x7e,0x97,0x60,0xea,0x20,0xb6} };

	return g;
}

//static
const GUID& MullvadGuids::Filter_Baseline_PermitEndpoint_Ipv6_9() {
	// {5E36E8E4-7D56-41C2-8655-5D09EECFA784}
	static const GUID g = { 0x5e36e8e4,0x7d56,0x41c2,{0x86,0x55,0x5d,0x09,0xee,0xcf,0xa7,0x84} };

	return g;
}

//static
const GUID& MullvadGuids::Filter_Baseline_PermitEndpoint_Ipv6_10() {
	// {6A3345C4-91B3-4CFB-B26F-727C81FEBB39}
	static const GUID g = { 0x6a3345c4,0x91b3,0x4cfb,{0xb2,0x6f,0x72,0x7c,0x81,0xfe,0xbb,0x39} };

	return g;
}

//static
const GUID& MullvadGuids::Filter_Baseline_PermitEndpoint_Ipv6_11() {
	// {769D1786-08D9-4EAD-902A-957DEAC26B40}
	static const GUID g = { 0x769d1786,0x08d9,0x4ead,{0x90,0x2a,0x95,0x7d,0xea,0xc2,0x6b,0x40} };

	return g;
}

//static
const GUID& MullvadGuids::Filter_Baseline_PermitEndpoint_Ipv6_12() {
	// {860A7F65-C35A-4FD0-A725-D34BE2551774}
	static const GUID g = { 0x860a7f65,0xc35a,0x4fd0,{0xa7,0x25,0xd3,0x4b,0xe2,0x55,0x17,0x74} };

	return g;
}

//static
const GUID& MullvadGuids::Filter_Baseline_PermitEndpoint_Ipv6_13() {
	// {D9A1E8C1-5F27-4F60-9B7E-43B7D8B85B37}
	static const GUID g = { 0xd9a1e8c1,0x5f27,0x4f60,{0x9b,0x7e,0x43,0xb7,0xd8,0xb8,0x5b,0x37} };

	return g;
}

//static
const GUID& MullvadGuids::Filter_Baseline_PermitEndpoint_Ipv6_14() {
	// {F2B6D4E9-0D8A-4E3C-91A5-9D0C9E3E7F2A}
	static const GUID g = { 0xf2b6d4e9,0x0d8a,0x4e3c,{0x91,0xa5,0x9d,0x0c,0x9e,0x3e,0x7f,0x2a} };

	return g;
}

//static
const GUID& MullvadGuids::Filter_Baseline_PermitEndpoint_Ipv6_15() {
	// {8C3E4A27-2A0B-4F1D-A3F3-0E8F9B5C6D71}
	static const GUID g = { 0x8c3e4a27,0x2a0b,0x4f1d,{0xa3,0xf3,0x0e,0x8f,0x9b,0x5c,0x6d,0x71} };

	return g;
}

//static
const GUID& MullvadGuids::Filter_Baseline_PermitEndpoint_Ipv6_16() {
	// {1A7FD3B4-6E2C-4E72-B1D2-5C0A7E9F3B84}
	static const GUID g = { 0x1a7fd3b4,0x6e2c,0x4e72,{0xb1,0xd2,0x5c,0x0a,0x7e,0x9f,0x3b,0x84} };

	return g;
}

//static
const GUID& MullvadGuids::Filter_Baseline_PermitEndpoint_Ipv6_17() {
	// {4E2B9A70-9F13-4E08-AB8F-2D4C7A1E5F90}
	static const GUID g = { 0x4e2b9a70,0x9f13,0x4e08,{0xab,0x8f,0x2d,0x4c,0x7a,0x1e,0x5f,0x90} };

	return g;
}

//static
const GUID& MullvadGuids::Filter_Baseline_PermitEndpoint_Ipv6_18() {
	// {A1C4D7E2-3B58-4F9A-8C71-6E2F1D0A3B9C}
	static const GUID g = { 0xa1c4d7e2,0x3b58,0x4f9a,{0x8c,0x71,0x6e,0x2f,0x1d,0x0a,0x3b,0x9c} };

	return g;
}

//static
const GUID& MullvadGuids::Filter_Baseline_PermitEndpoint_Ipv6_19() {
	// {B3E7A4C9-8D2F-4C61-90FA-0B7E5C2D1A46}
	static const GUID g = { 0xb3e7a4c9,0x8d2f,0x4c61,{0x90,0xfa,0x0b,0x7e,0x5c,0x2d,0x1a,0x46} };

	return g;
}

//static
const GUID& MullvadGuids::Filter_Baseline_PermitEndpoint_Ipv6_20() {
	// {0C5F8A71-4D2E-4B9C-AD17-2F6B3E9C5A80}
	static const GUID g = { 0x0c5f8a71,0x4d2e,0x4b9c,{0xad,0x17,0x2f,0x6b,0x3e,0x9c,0x5a,0x80} };

	return g;
}

//static
const GUID& MullvadGuids::Filter_Baseline_PermitEndpoint_Ipv6_21() {
	// {66E91C04-6B1C-4A75-8F91-2C6A54E3B7D2}
	static const GUID g = { 0x66e91c04,0x6b1c,0x4a75,{0x8f,0x91,0x2c,0x6a,0x54,0xe3,0xb7,0xd2} };

	return g;
}

//static
const GUID& MullvadGuids::Filter_Baseline_PermitEndpoint_Ipv6_22() {
	// {2B4F9E85-1D3A-4984-B2E0-7C51AD9F3486}
	static const GUID g = { 0x2b4f9e85,0x1d3a,0x4984,{0xb2,0xe0,0x7c,0x51,0xad,0x9f,0x34,0x86} };

	return g;
}

//static
const GUID& MullvadGuids::Filter_Baseline_PermitEndpoint_Ipv6_23() {
	// {9D0A3C7B-7E42-4C2E-85B3-FA104C2D7E65}
	static const GUID g = { 0x9d0a3c7b,0x7e42,0x4c2e,{0x85,0xb3,0xfa,0x10,0x4c,0x2d,0x7e,0x65} };

	return g;
}

//static
const GUID& MullvadGuids::Filter_Baseline_PermitEndpoint_Ipv6_24() {
	// {3CF1A8B2-2D77-4F6D-A2B9-1E5C0F9A7D34}
	static const GUID g = { 0x3cf1a8b2,0x2d77,0x4f6d,{0xa2,0xb9,0x1e,0x5c,0x0f,0x9a,0x7d,0x34} };

	return g;
}

//static
const GUID &MullvadGuids::Filter_Baseline_PermitVpnTunnel_Entry_Outbound_Ipv4_1()
{
	// {BCECE8D7-2BAA-40CE-A7E9-5A4044E24883}
	static const GUID g = { 0xbcece8d7,0x2baa,0x40ce,{0xa7,0xe9,0x5a,0x40,0x44,0xe2,0x48,0x83} };

	return g;
}

//static
const GUID &MullvadGuids::Filter_Baseline_PermitVpnTunnel_Entry_Outbound_Ipv6_1()
{
	// {0DBD1D20-112E-4B56-946D-6AB3DAB722C9}
	static const GUID g = { 0x0dbd1d20,0x112e,0x4b56,{0x94,0x6d,0x6a,0xb3,0xda,0xb7,0x22,0xc9} };

	return g;
}

//static
const GUID &MullvadGuids::Filter_Baseline_PermitVpnTunnel_Entry_Outbound_Ipv4_2()
{
	// {DCA44438-7942-4215-BD11-30DAE8EE0E03}
	static const GUID g = { 0xdca44438,0x7942,0x4215,{0xbd,0x11,0x30,0xda,0xe8,0xee,0x0e,0x03} };

	return g;
}

//static
const GUID & MullvadGuids::Filter_Baseline_PermitVpnTunnel_Entry_Outbound_Ipv6_2()
{
	// {36862FAF-4AC0-4852-95A1-FF314F9F2F5B}
	static const GUID g = { 0x36862faf,0x4ac0,0x4852,{0x95,0xa1,0xff,0x31,0x4f,0x9f,0x2f,0x5b} };

	return g;
}

//static
const GUID &MullvadGuids::Filter_Baseline_PermitVpnTunnel_Exit_Outbound_Ipv4_1()
{
	// {C593D84F-9F07-429A-9B78-CE6CB4249EFC}
	static const GUID g = { 0xc593d84f,0x9f07,0x429a,{0x9b,0x78,0xce,0x6c,0xb4,0x24,0x9e,0xfc} };

	return g;
}

//static
const GUID &MullvadGuids::Filter_Baseline_PermitVpnTunnel_Exit_Outbound_Ipv6_1()
{
	// {04A39B8D-03DC-4C93-AE62-E3D6BA4178F3}
	static const GUID g = { 0x04a39b8d,0x03dc,0x4c93,{0xae,0x62,0xe3,0xd6,0xba,0x41,0x78,0xf3} };

	return g;
}

//static
const GUID &MullvadGuids::Filter_Baseline_PermitVpnTunnel_Exit_Outbound_Ipv4_2()
{
	// {67EE5B14-C670-47B7-B6C5-E9EE234C715E}
	static const GUID g = { 0x67ee5b14,0xc670,0x47b7,{0xb6,0xc5,0xe9,0xee,0x23,0x4c,0x71,0x5e} };

	return g;
}

//static
const GUID &MullvadGuids::Filter_Baseline_PermitVpnTunnel_Exit_Outbound_Ipv6_2()
{
	// {2C632BDB-F1AB-42C7-A7FE-91CE2DF74E9F}
	static const GUID g = { 0x2c632bdb,0xf1ab,0x42c7,{0xa7,0xfe,0x91,0xce,0x2d,0xf7,0x4e,0x9f} };

	return g;
}

//static
const GUID &MullvadGuids::Filter_Baseline_PermitVpnTunnelService_Entry_Ipv4_1()
{
	// {4A83F108-7008-4510-8EE3-900A7495CAAB}
	static const GUID g = { 0x4a83f108,0x7008,0x4510,{0x8e,0xe3,0x90,0x0a,0x74,0x95,0xca,0xab} };

	return g;
}

//static
const GUID &MullvadGuids::Filter_Baseline_PermitVpnTunnelService_Entry_Ipv6_1()
{
	// {652E1F33-4E01-4F27-B0B9-74912AA8F110}
	static const GUID g = { 0x652e1f33,0x4e01,0x4f27,{0xb0,0xb9,0x74,0x91,0x2a,0xa8,0xf1,0x10} };

	return g;
}

//static
const GUID &MullvadGuids::Filter_Baseline_PermitVpnTunnelService_Entry_Ipv4_2()
{
	// {0F2F41E9-6403-4A35-B9D0-D1784E400869}
	static const GUID g = { 0x0f2f41e9,0x6403,0x4a35,{0xb9,0xd0,0xd1,0x78,0x4e,0x40,0x08,0x69} };

	return g;
}

//static
const GUID &MullvadGuids::Filter_Baseline_PermitVpnTunnelService_Entry_Ipv6_2()
{
	// {D83633A3-E391-4391-AA85-8186B95DC363}
	static const GUID g = { 0xd83633a3,0xe391,0x4391,{0xaa,0x85,0x81,0x86,0xb9,0x5d,0xc3,0x63} };

	return g;
}

//static
const GUID &MullvadGuids::Filter_Baseline_PermitVpnTunnelService_Exit_Ipv4_1()
{
	// {9D857D88-211D-41DC-8A4C-1BC73474173C}
	static const GUID g = { 0x9d857d88,0x211d,0x41dc,{0x8a,0x4c,0x1b,0xc7,0x34,0x74,0x17,0x3c} };

	return g;
}

//static
const GUID &MullvadGuids::Filter_Baseline_PermitVpnTunnelService_Exit_Ipv6_1()
{
	// {32798A35-721E-4313-90EF-BC4CE42B00B3}
	static const GUID g = { 0x32798a35,0x721e,0x4313,{0x90,0xef,0xbc,0x4c,0xe4,0x2b,0x00,0xb3} };

	return g;
}

//static
const GUID &MullvadGuids::Filter_Baseline_PermitVpnTunnelService_Exit_Ipv4_2()
{
	// {BD6B5856-5F51-45E9-A4EB-B18202826191}
	static const GUID g = { 0xbd6b5856,0x5f51,0x45e9,{0xa4,0xeb,0xb1,0x82,0x02,0x82,0x61,0x91} };

	return g;
}

//static
const GUID &MullvadGuids::Filter_Baseline_PermitVpnTunnelService_Exit_Ipv6_2()
{
	// {131E52D0-502D-436F-B1A2-88A979CCBF9F}
	static const GUID g = { 0x131e52d0,0x502d,0x436f,{0xb1,0xa2,0x88,0xa9,0x79,0xcc,0xbf,0x9f} };

	return g;
}

//static
const GUID &MullvadGuids::Filter_Baseline_PermitNdp_Outbound_Router_Solicitation()
{
	// {755A4486-3CF5-4F5D-9308-AD1A3F4A7DE4}
	static const GUID g = { 0x755a4486,0x3cf5,0x4f5d,{0x93,0x08,0xad,0x1a,0x3f,0x4a,0x7d,0xe4} };

	return g;
}

//static
const GUID &MullvadGuids::Filter_Baseline_PermitNdp_Inbound_Router_Advertisement()
{
	// {43C954BA-3739-4762-B3DD-F6FA94B31847}
	static const GUID g = { 0x43c954ba,0x3739,0x4762,{0xb3,0xdd,0xf6,0xfa,0x94,0xb3,0x18,0x47} };

	return g;
}

//static
const GUID &MullvadGuids::Filter_Baseline_PermitNdp_Outbound_Neighbor_Solicitation()
{
	// {FEA40503-ADC7-450C-9B66-5CB0691FDEB4}
	static const GUID g = { 0xfea40503,0xadc7,0x450c,{0x9b,0x66,0x5c,0xb0,0x69,0x1f,0xde,0xb4} };

	return g;
}

//static
const GUID &MullvadGuids::Filter_Baseline_PermitNdp_Inbound_Neighbor_Solicitation()
{
	// {843D33CC-99CB-4E67-A1D3-BD5744EFAB61}
	static const GUID g = { 0x843d33cc,0x99cb,0x4e67,{0xa1,0xd3,0xbd,0x57,0x44,0xef,0xab,0x61} };

	return g;
}

//static
const GUID &MullvadGuids::Filter_Baseline_PermitNdp_Outbound_Neighbor_Advertisement()
{
	// {4C3F711E-D479-4FB2-81D2-1CE3A8D39128}
	static const GUID g = { 0x4c3f711e,0xd479,0x4fb2,{0x81,0xd2,0x1c,0xe3,0xa8,0xd3,0x91,0x28} };

	return g;
}

//static
const GUID &MullvadGuids::Filter_Baseline_PermitNdp_Inbound_Neighbor_Advertisement()
{
	// {1BFBA8E5-FBF5-4D81-B7E5-34B211934F7E}
	static const GUID g = { 0x1bfba8e5,0xfbf5,0x4d81,{0xb7,0xe5,0x34,0xb2,0x11,0x93,0x4f,0x7e} };

	return g;
}

//static
const GUID &MullvadGuids::Filter_Baseline_PermitNdp_Inbound_Redirect()
{
	// {CB455186-0ED9-493C-B023-BB3810A79CF9}
	static const GUID g = { 0xcb455186,0x0ed9,0x493c,{0xb0,0x23,0xbb,0x38,0x10,0xa7,0x9c,0xf9} };

	return g;
}

//static
const GUID &MullvadGuids::Filter_Baseline_PermitDns_Outbound_Ipv4()
{
	// {A1259109-FC57-47F8-8FDA-799903D90D39}
	static const GUID g = { 0xa1259109,0xfc57,0x47f8,{0x8f,0xda,0x79,0x99,0x03,0xd9,0x0d,0x39} };

	return g;
}

//static
const GUID &MullvadGuids::Filter_Baseline_PermitDns_Outbound_Ipv6()
{
	// {FD90A07D-A244-4FAF-BD6D-26B97E9B2893}
	static const GUID g = { 0xfd90a07d,0xa244,0x4faf,{0xbd,0x6d,0x26,0xb9,0x7e,0x9b,0x28,0x93} };

	return g;
}

//static
const GUID &MullvadGuids::Filter_Dns_BlockAll_Outbound_Ipv4()
{
	// {6DA3AD59-4217-42F8-A08D-016A76FEB2BD}
	static const GUID g = { 0x6da3ad59,0x4217,0x42f8,{0xa0,0x8d,0x01,0x6a,0x76,0xfe,0xb2,0xbd} };

	return g;
}

//static
const GUID &MullvadGuids::Filter_Dns_BlockAll_Outbound_Ipv6()
{
	// {067B590E-7845-4B87-A970-C7AE847A386A}
	static const GUID g = { 0x067b590e,0x7845,0x4b87,{0xa9,0x70,0xc7,0xae,0x84,0x7a,0x38,0x6a} };

	return g;
}

//static
const GUID &MullvadGuids::Filter_Dns_PermitNonTunnel_Outbound_Ipv4()
{
	// {69DCB3D4-FA52-43A5-B219-80CA48AF4C5C}
	static const GUID g = { 0x69dcb3d4,0xfa52,0x43a5,{0xb2,0x19,0x80,0xca,0x48,0xaf,0x4c,0x5c} };

	return g;
}

//static
const GUID &MullvadGuids::Filter_Dns_PermitNonTunnel_Outbound_Ipv6()
{
	// {E4CBCF1F-CEBC-44F9-84CA-E05A86C371AD}
	static const GUID g = { 0xe4cbcf1f,0xcebc,0x44f9,{0x84,0xca,0xe0,0x5a,0x86,0xc3,0x71,0xad} };

	return g;
}

//static
const GUID &MullvadGuids::Filter_Dns_PermitTunnel_Outbound_Ipv4()
{
	// {C0B4407B-0ECE-4C0B-A333-84F68BAE3E37}
	static const GUID g = { 0xc0b4407b,0x0ece,0x4c0b,{0xa3,0x33,0x84,0xf6,0x8b,0xae,0x3e,0x37} };

	return g;
}

//static
const GUID &MullvadGuids::Filter_Dns_PermitTunnel_Outbound_Ipv6()
{
	// {C59E6976-212E-4233-93C2-C51F941D7D65}
	static const GUID g = { 0xc59e6976,0x212e,0x4233,{0x93,0xc2,0xc5,0x1f,0x94,0x1d,0x7d,0x65} };

	return g;
}

//static
const GUID &MullvadGuids::Filter_Dns_PermitLoopback_Outbound_Ipv4()
{
	// {A9A29810-61A4-4331-A441-A20F51D2B45A}
	static const GUID g = { 0xa9a29810,0x61a4,0x4331,{0xa4,0x41,0xa2,0x0f,0x51,0xd2,0xb4,0x5a} };

	return g;
}

//static
const GUID &MullvadGuids::Filter_Dns_PermitLoopback_Outbound_Ipv6()
{
	// {3ED5BA1D-C39D-431A-8D51-85E915EBA7FA}
	static const GUID g = { 0x3ed5ba1d,0xc39d,0x431a,{0x8d,0x51,0x85,0xe9,0x15,0xeb,0xa7,0xfa} };

	return g;
}
