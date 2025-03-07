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

namespace {
	// Maximum number of allowed endpoint per IP protocol version.
	static const uint32_t MAX_ALLOWED_ENDPOINTS = 4;

	static const GUID ENDPOINT_IPV4_GUIDS[MAX_ALLOWED_ENDPOINTS] = { 
		MullvadGuids::Filter_Baseline_PermitEndpoint_Ipv4_1(),
		MullvadGuids::Filter_Baseline_PermitEndpoint_Ipv4_2(),
		MullvadGuids::Filter_Baseline_PermitEndpoint_Ipv4_3(),
		MullvadGuids::Filter_Baseline_PermitEndpoint_Ipv4_4(),
	};

	static const GUID ENDPOINT_IPV6_GUIDS[MAX_ALLOWED_ENDPOINTS] = {
		MullvadGuids::Filter_Baseline_PermitEndpoint_Ipv6_1(),
		MullvadGuids::Filter_Baseline_PermitEndpoint_Ipv6_2(),
		MullvadGuids::Filter_Baseline_PermitEndpoint_Ipv6_3(),
		MullvadGuids::Filter_Baseline_PermitEndpoint_Ipv6_4(),
	};

} // anonymous namespace

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

	for (auto endpoint: m_endpoints) {
		switch (endpoint.ip.type()) {
			case wfp::IpAddress::Type::Ipv4:
				if (!AddIpv4EndpointFilter(endpoint, ENDPOINT_IPV4_GUIDS[ipv4Count], objectInstaller)) {
					return false;
				}

				if (ipv4Count++ == MAX_ALLOWED_ENDPOINTS) {
					THROW_ERROR("Exceeded max allowed endpoints (IPv4)");
				}

				break;

			case wfp::IpAddress::Type::Ipv6:
				if (!AddIpv6EndpointFilter(endpoint, ENDPOINT_IPV6_GUIDS[ipv6Count], objectInstaller)) {
					return false;
				}

				if (ipv6Count++ == MAX_ALLOWED_ENDPOINTS) {
					THROW_ERROR("Exceeded max allowed endpoints (IPv6)");
				}

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

	for (const auto client : endpoint.clients) {
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

	for (const auto client : endpoint.clients) {
		conditionBuilder.add_condition(std::make_unique<ConditionApplication>(client));
	}

	return objectInstaller.addFilter(filterBuilder, conditionBuilder);
}

}
