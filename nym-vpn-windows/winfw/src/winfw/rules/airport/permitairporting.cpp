#include "stdafx.h"
#include "permitairporting.h"
#include <winfw/mullvadguids.h>
#include <libwfp/filterbuilder.h>
#include <libwfp/conditionbuilder.h>
#include <libwfp/conditions/conditionip.h>
#include <libcommon/error.h>

using namespace wfp::conditions;

namespace rules::airporting
{

PermitAirporting::PermitAirporting(std::vector<wfp::IpNetwork> networks)
{
	for (auto &network : networks)
	{
		if (network.type() == wfp::IpNetwork::Ipv4)
		{
			m_ipv4Networks.push_back(network);
		}
		else
		{
			m_ipv6Networks.push_back(network);
		}
	}
}

bool PermitAirporting::apply(IObjectInstaller &objectInstaller)
{
	return applyIpv4(objectInstaller) && applyIpv6(objectInstaller);
}

bool PermitAirporting::applyIpv4(IObjectInstaller &objectInstaller) const
{
	if (m_ipv4Networks.empty())
	{
		return true;
	}

	const auto &guidPool = MullvadGuids::Filter_Airport_PermitAirporting_Ipv4;
	const size_t poolSize = MullvadGuids::Num_Airport_PermitAirporting_Ipv4_Filters;

	// Calculate number of batches needed
	const size_t numBatches = (m_ipv4Networks.size() + MaxNetworksPerFilter - 1) / MaxNetworksPerFilter;
	if (numBatches > poolSize)
	{
		THROW_ERROR("Exceeded max allowed airporting networks (IPv4)");
	}

	size_t networkIndex = 0;
	size_t guidIndex = 0;

	while (networkIndex < m_ipv4Networks.size())
	{
		wfp::FilterBuilder filterBuilder;

		filterBuilder
			.key(guidPool[guidIndex++])
			.name(L"Permit outbound connections to airporting network (IPv4)")
			.description(L"This filter is part of a rule that permits traffic to bypass the VPN tunnel")
			.provider(MullvadGuids::Provider())
			.layer(FWPM_LAYER_ALE_AUTH_CONNECT_V4)
			.sublayer(MullvadGuids::SublayerAirporting())
			.weight(wfp::FilterBuilder::WeightClass::Max)
			.permit();

		wfp::ConditionBuilder conditionBuilder(FWPM_LAYER_ALE_AUTH_CONNECT_V4);

		const size_t batchEnd = min(networkIndex + MaxNetworksPerFilter, m_ipv4Networks.size());

		while (networkIndex < batchEnd)
		{
			conditionBuilder.add_condition(ConditionIp::Remote(m_ipv4Networks[networkIndex++]));
		}

		if (!objectInstaller.addFilter(filterBuilder, conditionBuilder))
		{
			return false;
		}
	}

	return true;
}

bool PermitAirporting::applyIpv6(IObjectInstaller &objectInstaller) const
{
	if (m_ipv6Networks.empty())
	{
		return true;
	}

	const auto &guidPool = MullvadGuids::Filter_Airport_PermitAirporting_Ipv6;
	const size_t poolSize = MullvadGuids::Num_Airport_PermitAirporting_Ipv6_Filters;

	// Calculate number of batches needed
	const size_t numBatches = (m_ipv6Networks.size() + MaxNetworksPerFilter - 1) / MaxNetworksPerFilter;
	if (numBatches > poolSize)
	{
		THROW_ERROR("Exceeded max allowed airporting networks (IPv6)");
	}

	size_t networkIndex = 0;
	size_t guidIndex = 0;

	while (networkIndex < m_ipv6Networks.size())
	{
		wfp::FilterBuilder filterBuilder;

		filterBuilder
			.key(guidPool[guidIndex++])
			.name(L"Permit outbound connections to airporting network (IPv6)")
			.description(L"This filter is part of a rule that permits traffic to bypass the VPN tunnel")
			.provider(MullvadGuids::Provider())
			.layer(FWPM_LAYER_ALE_AUTH_CONNECT_V6)
			.sublayer(MullvadGuids::SublayerAirporting())
			.weight(wfp::FilterBuilder::WeightClass::Max)
			.permit();

		wfp::ConditionBuilder conditionBuilder(FWPM_LAYER_ALE_AUTH_CONNECT_V6);

		const size_t batchEnd = min(networkIndex + MaxNetworksPerFilter, m_ipv6Networks.size());

		while (networkIndex < batchEnd)
		{
			conditionBuilder.add_condition(ConditionIp::Remote(m_ipv6Networks[networkIndex++]));
		}

		if (!objectInstaller.addFilter(filterBuilder, conditionBuilder))
		{
			return false;
		}
	}

	return true;
}

}
