#pragma once

#include <winfw/rules/ifirewallrule.h>
#include <winfw/winfw.h>
#include <libwfp/ipaddress.h>
#include <string>
#include <optional>

namespace rules::baseline
{

class PermitVpnTunnel : public IFirewallRule
{
public:

	enum InterfaceType {
		Entry,
		Exit,
	};

	struct Endpoint {
		wfp::IpAddress ip;
		uint16_t port;
		WinFwProtocol protocol;
	};

	struct Endpoints {
		Endpoint entryEndpoint;
		std::optional<Endpoint> exitEndpoint;
	};

	PermitVpnTunnel(
		const InterfaceType interfaceType,
		const std::wstring &tunnelInterfaceAlias,
		const std::optional<Endpoints> &potentialEndpoints
	);
	
	bool apply(IObjectInstaller &objectInstaller) override;

private:
	bool ApplyForEntryInterface(IObjectInstaller& objectInstaller);
	bool ApplyForExitInterface(IObjectInstaller& objectInstaller);
	bool AddEndpointFilter(const std::optional<Endpoint> &endpoint, const GUID &ipv4Guid, const GUID &ipv6Guid, IObjectInstaller &objectInstaller);

	const InterfaceType m_interfaceType;
	const std::wstring m_tunnelInterfaceAlias;
	const std::optional<Endpoints> m_potentialEndpoints;
};

}
