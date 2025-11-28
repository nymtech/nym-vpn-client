#include "stdafx.h"
#include "permitendpoint.h"
#include <winfw/mullvadguids.h>
#include <winfw/rules/shared.h>
#include <libwfp/filterbuilder.h>
#include <libwfp/conditionbuilder.h>
#include <libwfp/conditions/conditionprotocol.h>
#include <libwfp/conditions/conditionip.h>
#include <libwfp/conditions/conditionport.h>
#include <libwfp/conditions/conditionapplication.h>
#include <libcommon/error.h>

using namespace wfp::conditions;

namespace rules::baseline
{

PermitEndpoint::PermitEndpoint(const std::vector<Endpoint> endpoints)
	: m_endpoints(endpoints)
{
}

bool PermitEndpoint::apply(IObjectInstaller &objectInstaller)
{
	//
	// Permit outbound connections to endpoint.
	//

	uint32_t ipv4Count = 0;
	uint32_t ipv6Count = 0;

	for (auto &endpoint: m_endpoints) {
		switch (endpoint.ip.type()) {
			case wfp::IpAddress::Type::Ipv4:
				if (ipv4Count == MullvadGuids::Num_Baseline_PermitEndpoint_Ipv4_Filters) {
					THROW_ERROR("Exceeded max allowed endpoints (IPv4)");
				}

				if (!AddIpv4EndpointFilter(endpoint, MullvadGuids::Filter_Baseline_PermitEndpoint_Ipv4[ipv4Count], objectInstaller)) {
					return false;
				}

				ipv4Count++;

				break;

			case wfp::IpAddress::Type::Ipv6:
				if (ipv6Count == MullvadGuids::Num_Baseline_PermitEndpoint_Ipv6_Filters) {
					THROW_ERROR("Exceeded max allowed endpoints (IPv6)");
				}

				if (!AddIpv6EndpointFilter(endpoint, MullvadGuids::Filter_Baseline_PermitEndpoint_Ipv6[ipv6Count], objectInstaller)) {
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

bool PermitEndpoint::AddIpv4EndpointFilter(const Endpoint &endpoint, const GUID &ipv4Guid, IObjectInstaller &objectInstaller) 
{
	wfp::FilterBuilder filterBuilder;

	filterBuilder
		.key(ipv4Guid)
		.name(L"Permit outbound connections to a given endpoint (IPv4)")
		.description(L"This filter is part of a rule that permits traffic to a specific endpoint")
		.provider(MullvadGuids::Provider())
		.layer(FWPM_LAYER_ALE_AUTH_CONNECT_V4)
		.sublayer(MullvadGuids::SublayerBaseline())
		.weight(wfp::FilterBuilder::WeightClass::Max)
		.permit();

	wfp::ConditionBuilder conditionBuilder(FWPM_LAYER_ALE_AUTH_CONNECT_V4);

	conditionBuilder.add_condition(ConditionIp::Remote(endpoint.ip));
	conditionBuilder.add_condition(ConditionPort::Remote(endpoint.port));
	conditionBuilder.add_condition(CreateProtocolCondition(endpoint.protocol));

	for (const auto &client : endpoint.clients) {
		conditionBuilder.add_condition(std::make_unique<ConditionApplication>(client));
	}

	return objectInstaller.addFilter(filterBuilder, conditionBuilder);
}

bool PermitEndpoint::AddIpv6EndpointFilter(const Endpoint &endpoint, const GUID &ipv6Guid, IObjectInstaller &objectInstaller) 
{
	wfp::FilterBuilder filterBuilder;

	filterBuilder
		.key(ipv6Guid)
		.name(L"Permit outbound connections to a given endpoint (IPv6)")
		.description(L"This filter is part of a rule that permits traffic to a specific endpoint")
		.provider(MullvadGuids::Provider())
		.layer(FWPM_LAYER_ALE_AUTH_CONNECT_V6)
		.sublayer(MullvadGuids::SublayerBaseline())
		.weight(wfp::FilterBuilder::WeightClass::Max)
		.permit();

	wfp::ConditionBuilder conditionBuilder(FWPM_LAYER_ALE_AUTH_CONNECT_V6);

	conditionBuilder.add_condition(ConditionIp::Remote(endpoint.ip));
	conditionBuilder.add_condition(ConditionPort::Remote(endpoint.port));
	conditionBuilder.add_condition(CreateProtocolCondition(endpoint.protocol));

	for (const auto &client : endpoint.clients) {
		conditionBuilder.add_condition(std::make_unique<ConditionApplication>(client));
	}

	return objectInstaller.addFilter(filterBuilder, conditionBuilder);
}

}
