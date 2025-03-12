#pragma once

#include <winfw/rules/ifirewallrule.h>
#include <winfw/winfw.h>
#include <libwfp/ipaddress.h>
#include <string>

namespace rules::multi
{

class PermitVpnRelay : public IFirewallRule
{
public:

	enum class Sublayer
	{
		Baseline,
		Dns
	};

	struct Endpoint {
		wfp::IpAddress ip;
		uint16_t port;
		WinFwProtocol protocol;
		std::vector<std::wstring> clients;
		Sublayer sublayer;
	};

	PermitVpnRelay(std::vector<Endpoint> endpoints);
	
	bool apply(IObjectInstaller &objectInstaller) override;

private:

	bool AddIpv4RelayFilter(const Endpoint& endpoint, const GUID& ipv4Guid, IObjectInstaller& objectInstaller);
	bool AddIpv6RelayFilter(const Endpoint& endpoint, const GUID& ipv6Guid, IObjectInstaller& objectInstaller);

	const std::vector<Endpoint> m_endpoints;
};

}
