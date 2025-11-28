#include "stdafx.h"
#include "permitvpnrelay.h"
#include <winfw/mullvadguids.h>
#include <winfw/winfw.h>
#include <winfw/rules/shared.h>
#include <libwfp/filterbuilder.h>
#include <libwfp/conditionbuilder.h>
#include <libwfp/conditions/conditionprotocol.h>
#include <libwfp/conditions/conditionip.h>
#include <libwfp/conditions/conditionport.h>
#include <libwfp/conditions/conditionapplication.h>
#include <libcommon/error.h>

using namespace wfp::conditions;

namespace rules::multi
{

namespace
{

const GUID &TranslateSublayer(PermitVpnRelay::Sublayer sublayer)
{
	switch (sublayer)
	{
		case PermitVpnRelay::Sublayer::Baseline: return MullvadGuids::SublayerBaseline();
		case PermitVpnRelay::Sublayer::Dns: return MullvadGuids::SublayerDns();
		default:
		{
			THROW_ERROR("Missing case handler in switch clause");
		}
	};
}

} // anonymous namespace

PermitVpnRelay::PermitVpnRelay(std::vector<Endpoint> endpoints)
	: m_endpoints(endpoints)
{
}

bool PermitVpnRelay::apply(IObjectInstaller &objectInstaller)
{
	//
	// Permit outbound connections to relay.
	//

	uint32_t ipv4Count = 0;
	uint32_t ipv6Count = 0;

	for (auto endpoint : m_endpoints) {
		switch (endpoint.ip.type()) {
		case wfp::IpAddress::Type::Ipv4:
			if (ipv4Count == MullvadGuids::Num_Baseline_PermitVpnRelay_Ipv4_Filters) {
				THROW_ERROR("Exceeded max allowed endpoints (IPv4)");
            }

			if (!AddIpv4RelayFilter(endpoint, MullvadGuids::Filter_Baseline_PermitVpnRelay_Ipv4[ipv4Count], objectInstaller)) {
				return false;
			}

			ipv4Count++;

			break;

		case wfp::IpAddress::Type::Ipv6:
			if (ipv6Count == MullvadGuids::Num_Baseline_PermitVpnRelay_Ipv6_Filters) {
				THROW_ERROR("Exceeded max allowed endpoints (IPv6)");
			}

			if (!AddIpv6RelayFilter(endpoint, MullvadGuids::Filter_Baseline_PermitVpnRelay_Ipv6[ipv6Count], objectInstaller)) {
				return false;
			}

			ipv6Count++;

			break;

		default:
		{
			THROW_ERROR("Missing case handler in switch clause");
		}
		}
	}

	return true;
}

bool PermitVpnRelay::AddIpv4RelayFilter(const Endpoint& endpoint, const GUID& ipv4Guid, IObjectInstaller& objectInstaller) {
	wfp::FilterBuilder filterBuilder;

	filterBuilder
		.key(ipv4Guid)
		.name(L"Permit outbound connections to VPN relay")
		.description(L"This filter is part of a rule that permits communication with a VPN relay")
		.provider(MullvadGuids::Provider())
		.layer(FWPM_LAYER_ALE_AUTH_CONNECT_V4)
		.sublayer(TranslateSublayer(endpoint.sublayer))
		.weight(wfp::FilterBuilder::WeightClass::Medium)
		.permit();

	wfp::ConditionBuilder conditionBuilder(FWPM_LAYER_ALE_AUTH_CONNECT_V4);

	conditionBuilder.add_condition(ConditionIp::Remote(endpoint.ip));
	conditionBuilder.add_condition(ConditionPort::Remote(endpoint.port));
	conditionBuilder.add_condition(CreateProtocolCondition(endpoint.protocol));

	for (auto relayClient : endpoint.clients) {
		conditionBuilder.add_condition(std::make_unique<ConditionApplication>(relayClient));
	}

	return objectInstaller.addFilter(filterBuilder, conditionBuilder);
}

bool PermitVpnRelay::AddIpv6RelayFilter(const Endpoint& endpoint, const GUID& ipv6Guid, IObjectInstaller& objectInstaller) {
	wfp::FilterBuilder filterBuilder;

	filterBuilder
		.key(ipv6Guid)
		.name(L"Permit outbound connections to VPN relay")
		.description(L"This filter is part of a rule that permits communication with a VPN relay")
		.provider(MullvadGuids::Provider())
		.layer(FWPM_LAYER_ALE_AUTH_CONNECT_V6)
		.sublayer(TranslateSublayer(endpoint.sublayer))
		.weight(wfp::FilterBuilder::WeightClass::Medium)
		.permit();

	wfp::ConditionBuilder conditionBuilder(FWPM_LAYER_ALE_AUTH_CONNECT_V6);

	conditionBuilder.add_condition(ConditionIp::Remote(endpoint.ip));
	conditionBuilder.add_condition(ConditionPort::Remote(endpoint.port));
	conditionBuilder.add_condition(CreateProtocolCondition(endpoint.protocol));

	for (auto relayClient : endpoint.clients) {
		conditionBuilder.add_condition(std::make_unique<ConditionApplication>(relayClient));
	}

	return objectInstaller.addFilter(filterBuilder, conditionBuilder);
}

}
