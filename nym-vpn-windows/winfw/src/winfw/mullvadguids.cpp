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
	registry.insert(std::make_pair(WfpObjectType::Filter, Filter_Baseline_PermitVpnRelay_Ipv6_1()));
	registry.insert(std::make_pair(WfpObjectType::Filter, Filter_Baseline_PermitVpnRelay_Ipv4_2()));
	registry.insert(std::make_pair(WfpObjectType::Filter, Filter_Baseline_PermitVpnRelay_Ipv6_2()));
	registry.insert(std::make_pair(WfpObjectType::Filter, Filter_Baseline_PermitEndpoint_Ipv4_1()));
	registry.insert(std::make_pair(WfpObjectType::Filter, Filter_Baseline_PermitEndpoint_Ipv6_1()));
	registry.insert(std::make_pair(WfpObjectType::Filter, Filter_Baseline_PermitEndpoint_Ipv4_2()));
	registry.insert(std::make_pair(WfpObjectType::Filter, Filter_Baseline_PermitEndpoint_Ipv6_2()));
	registry.insert(std::make_pair(WfpObjectType::Filter, Filter_Baseline_PermitEndpoint_Ipv4_3()));
	registry.insert(std::make_pair(WfpObjectType::Filter, Filter_Baseline_PermitEndpoint_Ipv6_3()));
	registry.insert(std::make_pair(WfpObjectType::Filter, Filter_Baseline_PermitEndpoint_Ipv4_4()));
	registry.insert(std::make_pair(WfpObjectType::Filter, Filter_Baseline_PermitEndpoint_Ipv6_4()));
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
const GUID &MullvadGuids::Filter_Baseline_PermitEndpoint_Ipv6_1()
{
	// {C88C848F-2DF9-4908-944D-DE550CAD325E}
	static const GUID g = { 0xc88c848f,0x2df9,0x4908,{0x94,0x4d,0xde,0x55,0x0c,0xad,0x32,0x5e} };

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
const GUID &MullvadGuids::Filter_Baseline_PermitEndpoint_Ipv6_2()
{
	// {A8777D53-399B-418F-B24F-B03BAEABB68E}
	static const GUID g = { 0xa8777d53,0x399b,0x418f,{0xb2,0x4f,0xb0,0x3b,0xae,0xab,0xb6,0x8e} };

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
const GUID &MullvadGuids::Filter_Baseline_PermitEndpoint_Ipv6_3()
{
	// {ECE12F4D-EA16-4672-A128-43BE87A2D9C9}
	static const GUID g = { 0xece12f4d,0xea16,0x4672,{0xa1,0x28,0x43,0xbe,0x87,0xa2,0xd9,0xc9} };

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
const GUID &MullvadGuids::Filter_Baseline_PermitEndpoint_Ipv6_4()
{
	// {A6674EDA-3AA6-4937-B2DC-FAE0B1AE83BE}
	static const GUID g = { 0xa6674eda,0x3aa6,0x4937,{0xb2,0xdc,0xfa,0xe0,0xb1,0xae,0x83,0xbe} };

	return g;
}

const GUID &MullvadGuids::Filter_Baseline_PermitVpnTunnel_Entry_Outbound_Ipv4_1()
{
	// {BCECE8D7-2BAA-40CE-A7E9-5A4044E24883}
	static const GUID g = { 0xbcece8d7,0x2baa,0x40ce,{0xa7,0xe9,0x5a,0x40,0x44,0xe2,0x48,0x83} };

	return g;
}

const GUID &MullvadGuids::Filter_Baseline_PermitVpnTunnel_Entry_Outbound_Ipv6_1()
{
	// {0DBD1D20-112E-4B56-946D-6AB3DAB722C9}
	static const GUID g = { 0x0dbd1d20,0x112e,0x4b56,{0x94,0x6d,0x6a,0xb3,0xda,0xb7,0x22,0xc9} };

	return g;
}

const GUID &MullvadGuids::Filter_Baseline_PermitVpnTunnel_Entry_Outbound_Ipv4_2()
{
	// {DCA44438-7942-4215-BD11-30DAE8EE0E03}
	static const GUID g = { 0xdca44438,0x7942,0x4215,{0xbd,0x11,0x30,0xda,0xe8,0xee,0x0e,0x03} };

	return g;
}

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
