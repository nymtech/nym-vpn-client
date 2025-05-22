#include "stdafx.h"
#include "permitsecuredns.h"
#include <winfw/mullvadguids.h>
#include <winfw/rules/ports.h>
#include <winfw/rules/shared.h>
#include <libwfp/filterbuilder.h>
#include <libwfp/conditionbuilder.h>
#include <libwfp/conditions/conditionprotocol.h>
#include <libwfp/conditions/conditionip.h>
#include <libwfp/conditions/conditionport.h>
#include <libcommon/error.h>

using namespace wfp::conditions;

namespace rules::baseline
{

namespace {
    // Maximum number of allowed endpoint per IP protocol version.
    static const uint32_t MAX_ALLOWED_SECURE_DNS_ADDRESSES = 4;

    static const GUID SECURE_DNS_IPV4_GUIDS[MAX_ALLOWED_ENDPOINTS] = { 
        MullvadGuids::Filter_Baseline_PermitSecureDns_Ipv4_1(),
        MullvadGuids::Filter_Baseline_PermitSecureDns_Ipv4_2(),
        MullvadGuids::Filter_Baseline_PermitSecureDns_Ipv4_3(),
        MullvadGuids::Filter_Baseline_PermitSecureDns_Ipv4_4(),
    };

    static const GUID SECURE_DNS_IPV6_GUIDS[MAX_ALLOWED_ENDPOINTS] = {
        MullvadGuids::Filter_Baseline_PermitSecureDns_Ipv6_1(),
        MullvadGuids::Filter_Baseline_PermitSecureDns_Ipv6_2(),
        MullvadGuids::Filter_Baseline_PermitSecureDns_Ipv6_3(),
        MullvadGuids::Filter_Baseline_PermitSecureDns_Ipv6_4(),
    };

} // anonymous namespace

PermitSecureDns::PermitSecureDns(const std::vector<wfp::IpAddress> addresses) : m_addresses(addresses) {}

bool PermitSecureDns::apply(IObjectInstaller& objectInstaller) {

	//
	// Permit outbound connections to secure DNS addresses.
	//

	uint32_t ipv4Count = 0;
	uint32_t ipv6Count = 0;

	for (auto dns_address : m_addresses) {
		switch (dns_address.type()) {
		case wfp::IpAddress::Type::Ipv4:
			if (!AddIpv4EndpointFilter(dns_address, SECURE_DNS_IPV4_GUIDS[ipv4Count], objectInstaller)) {
				return false;
			}

			if (ipv4Count++ == MAX_ALLOWED_SECURE_DNS_ADDRESSES) {
				THROW_ERROR("Exceeded max allowed secure dns addresses (IPv4)");
			}

			break;

		case wfp::IpAddress::Type::Ipv6:
			if (!AddIpv6EndpointFilter(dns_address, SECURE_DNS_IPV6_GUIDS[ipv6Count], objectInstaller)) {
				return false;
			}

			if (ipv6Count++ == MAX_ALLOWED_SECURE_DNS_ADDRESSES) {
				THROW_ERROR("Exceeded max allowed secure dns addresses (IPv6)");
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


bool PermitSecureDns::AddIpv4EndpointFilter(const wfp::IpAddress &dns_address, const GUID& ipv4Guid, IObjectInstaller& objectInstaller)
{
	wfp::FilterBuilder filterBuilder;

	//
	// #1 Permit outbounds secure DNS, IPv4.
	//

	filterBuilder
		.key(ipv4Guid)
		.name(L"Permit outbound connections to secure DNS server (IPv4)")
		.description(L"This filter is part of a rule that permits outbound DNS")
		.provider(MullvadGuids::Provider())
		.layer(FWPM_LAYER_ALE_AUTH_CONNECT_V4)
		.sublayer(MullvadGuids::SublayerBaseline())
		.weight(wfp::FilterBuilder::WeightClass::Max)
		.permit();

	wfp::ConditionBuilder conditionBuilder(FWPM_LAYER_ALE_AUTH_CONNECT_V4);

	conditionBuilder.add_condition(ConditionIp::Remote(dns_address));

	conditionBuilder.add_condition(ConditionPort::Remote(DNS_OVER_HTTPS_PORT));
	conditionBuilder.add_condition(ConditionPort::Remote(DNS_OVER_TLS_PORT));

	conditionBuilder.add_condition(CreateProtocolCondition(WinFwProtocol::Tcp));

	return objectInstaller.addFilter(filterBuilder, conditionBuilder);
}

bool PermitSecureDns::AddIpv6EndpointFilter(const wfp::IpAddress& dns_address, const GUID& ipv6Guid, IObjectInstaller& objectInstaller)
{
	wfp::FilterBuilder filterBuilder;

	//
	// #2 Permit outbound secure DNS, IPv6.
	//

	filterBuilder
		.key(ipv6Guid)
		.name(L"Permit outbound connections to secure DNS server (IPv6)")
		.description(L"This filter is part of a rule that permits outbound DNS")
		.provider(MullvadGuids::Provider())
		.layer(FWPM_LAYER_ALE_AUTH_CONNECT_V6)
		.sublayer(MullvadGuids::SublayerBaseline())
		.weight(wfp::FilterBuilder::WeightClass::Max)
		.permit();

	wfp::ConditionBuilder conditionBuilder(FWPM_LAYER_ALE_AUTH_CONNECT_V6);

	conditionBuilder.add_condition(ConditionIp::Remote(dns_address));

	conditionBuilder.add_condition(ConditionPort::Remote(DNS_OVER_HTTPS_PORT));
	conditionBuilder.add_condition(ConditionPort::Remote(DNS_OVER_TLS_PORT));

	conditionBuilder.add_condition(CreateProtocolCondition(WinFwProtocol::Tcp));

	return objectInstaller.addFilter(filterBuilder, conditionBuilder);
}

}
