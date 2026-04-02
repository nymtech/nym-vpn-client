#pragma once

#include <winfw/rules/ifirewallrule.h>
#include <libwfp/ipnetwork.h>
#include <vector>

namespace rules::airporting
{

class PermitAirporting : public IFirewallRule
{
public:

	PermitAirporting(std::vector<wfp::IpNetwork> networks);

	bool apply(IObjectInstaller &objectInstaller) override;

private:

	static const size_t MaxNetworksPerFilter = 500;

	bool applyIpv4(IObjectInstaller &objectInstaller) const;
	bool applyIpv6(IObjectInstaller &objectInstaller) const;

	std::vector<wfp::IpNetwork> m_ipv4Networks;
	std::vector<wfp::IpNetwork> m_ipv6Networks;
};

}
