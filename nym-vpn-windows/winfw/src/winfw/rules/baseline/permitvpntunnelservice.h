#pragma once

#include <winfw/rules/ifirewallrule.h>
#include <winfw/rules/baseline/permitvpntunnel.h>
#include <winfw/winfw.h>
#include <libwfp/ipaddress.h>
#include <string>
#include <optional>

namespace rules::baseline
{

class PermitVpnTunnelService : public IFirewallRule
{
public:

	PermitVpnTunnelService(
		const PermitVpnTunnel::InterfaceType interfaceType,
		const std::wstring &tunnelInterfaceAlias,
		const std::optional<PermitVpnTunnel::Endpoints> &potentialEndpoints
	);

	bool apply(IObjectInstaller &objectInstaller) override;

private:
	bool ApplyForEntryInterface(IObjectInstaller& objectInstaller);
	bool ApplyForExitInterface(IObjectInstaller& objectInstaller);
	bool AddEndpointFilter(const std::optional<PermitVpnTunnel::Endpoint> &endpoint, const GUID &ipv4Guid, const GUID &ipv6Guid, IObjectInstaller &objectInstaller);

	const PermitVpnTunnel::InterfaceType m_interfaceType;
	const std::wstring m_tunnelInterfaceAlias;
	const std::optional<PermitVpnTunnel::Endpoints> m_potentialEndpoints;
};

}
