#pragma once

#include <winfw/rules/ifirewallrule.h>
#include <winfw/winfw.h>
#include <libwfp/ipaddress.h>
#include <vector>
#include <string>
#include <optional>

namespace rules::baseline
{

class PermitEndpoint : public IFirewallRule
{
public:

	struct Endpoint {
		wfp::IpAddress ip;
		uint16_t port;
		WinFwProtocol protocol;
		std::vector<std::wstring> clients;
	};

	PermitEndpoint(const std::vector<Endpoint> endpoints);
	
	bool apply(IObjectInstaller &objectInstaller) override;

private:
	bool AddIpv4EndpointFilter(const Endpoint &endpoint, const GUID &ipv4Guid, IObjectInstaller &objectInstaller);
	bool AddIpv6EndpointFilter(const Endpoint &endpoint, const GUID &ipv6Guid, IObjectInstaller &objectInstaller);

	const std::vector<Endpoint> m_endpoints;
};

}
