#pragma once

#include <winfw/rules/ifirewallrule.h>
#include <libwfp/ipaddress.h>
#include <vector>

namespace rules::baseline
{

class PermitSecureDns : public IFirewallRule
{
public:
	PermitSecureDns(const std::vector<wfp::IpAddress> addresses);

	bool apply(IObjectInstaller& objectInstaller) override;

private:
	bool AddIpv4EndpointFilter(const wfp::IpAddress& dns_address, const GUID& ipv4Guid, IObjectInstaller& objectInstaller);
	bool AddIpv6EndpointFilter(const wfp::IpAddress& dns_address, const GUID& ipv6Guid, IObjectInstaller& objectInstaller);

	const std::vector<wfp::IpAddress> m_addresses;
};

}
