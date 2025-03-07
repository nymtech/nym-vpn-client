#include "stdafx.h"
#include "fwcontext.h"
#include "mullvadobjects.h"
#include "objectpurger.h"
#include "rules/ifirewallrule.h"
#include "rules/ports.h"
#include "rules/baseline/blockall.h"
#include "rules/baseline/permitdhcp.h"
#include "rules/baseline/permitndp.h"
#include "rules/baseline/permitdhcpserver.h"
#include "rules/baseline/permitlan.h"
#include "rules/baseline/permitlanservice.h"
#include "rules/baseline/permitloopback.h"
#include "rules/baseline/permitvpntunnel.h"
#include "rules/baseline/permitvpntunnelservice.h"
#include "rules/baseline/permitdns.h"
#include "rules/baseline/permitendpoint.h"
#include "rules/dns/blockall.h"
#include "rules/dns/permitloopback.h"
#include "rules/dns/permittunnel.h"
#include "rules/dns/permitnontunnel.h"
#include "rules/multi/permitvpnrelay.h"
#include <libwfp/transaction.h>
#include <libwfp/filterengine.h>
#include <libcommon/error.h>
#include <functional>
#include <utility>

using namespace rules;

namespace
{

//
// Since the PermitLan rule doesn't specifically address DNS, it will allow DNS requests targeting
// a local resolver to leave the machine. From the local resolver the request will either be
// resolved from cache, or forwarded out onto the Internet.
//
// Therefore, we're unconditionally lifting all DNS traffic out of the baseline sublayer and restricting
// it in the DNS sublayer instead. The PermitDNS rule in the baseline sublayer accomplishes this.
//
// This has implications for the way the relay access is configured. In the regular case there
// is no issue: The PermitVpnRelay rule can be installed in the baseline sublayer.
//
// However, if the relay is running on the DNS port (53), it would be blocked unless the DNS
// sublayer permits this traffic. For this reason, whenever the relay is on port 53, the
// PermitVpnRelay rule has to be installed to the DNS sublayer instead of the baseline sublayer.
//
void AppendSettingsRules
(
	FwContext::Ruleset &ruleset,
	const WinFwSettings &settings
)
{
	if (settings.permitDhcp)
	{
		ruleset.emplace_back(std::make_unique<baseline::PermitDhcp>());
		ruleset.emplace_back(std::make_unique<baseline::PermitNdp>());
	}

	if (settings.permitLan)
	{
		ruleset.emplace_back(std::make_unique<baseline::PermitLan>());
		ruleset.emplace_back(std::make_unique<baseline::PermitLanService>());
		ruleset.emplace_back(baseline::PermitDhcpServer::WithExtent(baseline::PermitDhcpServer::Extent::IPv4Only));
	}

	//
	// DNS management
	//

	ruleset.emplace_back(std::make_unique<baseline::PermitDns>());
	ruleset.emplace_back(std::make_unique<dns::PermitLoopback>());
	ruleset.emplace_back(std::make_unique<dns::BlockAll>());
}

//
// Refer comment on `AppendSettingsRules`.
//
void AppendRelayRules
(
	FwContext::Ruleset &ruleset,
	const std::vector<WinFwAllowedEndpoint> &relays
)
{
	std::vector<multi::PermitVpnRelay::Endpoint> rule_endpoints;

	for (const auto& relay : relays)
	{
		std::vector<std::wstring> relayClients;
		if (relay.numClients > 0)
		{
			relayClients.reserve(relay.numClients);

			for (size_t relayClientIndex = 0; relayClientIndex < relay.numClients; ++relayClientIndex)
			{
				const auto& relayClient = relay.clients[relayClientIndex];
				if (relayClient != nullptr)
				{
					relayClients.push_back(relayClient);
				}
			}
		}

		auto sublayer =
			(
				DNS_SERVER_PORT == relay.endpoint.port
				? rules::multi::PermitVpnRelay::Sublayer::Dns
				: rules::multi::PermitVpnRelay::Sublayer::Baseline
				);

		rule_endpoints.emplace_back(multi::PermitVpnRelay::Endpoint {
			wfp::IpAddress(relay.endpoint.ip), 
			relay.endpoint.port,
			relay.endpoint.protocol,
			relayClients,
			sublayer
		});
	}

	ruleset.emplace_back(std::make_unique<multi::PermitVpnRelay>(rule_endpoints));
}

//
// Refer comment on `AppendSettingsRules`.
//
void AppendAllowedEndpointRules
(
	FwContext::Ruleset &ruleset,
	const std::vector<WinFwAllowedEndpoint> &endpoints
)
{
	std::vector<baseline::PermitEndpoint::Endpoint> rule_endpoints;

	for (const auto& endpoint : endpoints)
	{
		std::vector<std::wstring> clients;
		clients.reserve(endpoint.numClients);
		for (uint32_t i = 0; i < endpoint.numClients; i++) {
			clients.push_back(endpoint.clients[i]);
		}

		rule_endpoints.emplace_back(baseline::PermitEndpoint::Endpoint {
			wfp::IpAddress(endpoint.endpoint.ip), 
			endpoint.endpoint.port,
			endpoint.endpoint.protocol,
			clients
		});
	}

	ruleset.emplace_back(std::make_unique<baseline::PermitEndpoint>(rule_endpoints));
}

void AppendNetBlockedRules(FwContext::Ruleset &ruleset)
{
	ruleset.emplace_back(std::make_unique<baseline::BlockAll>());
	ruleset.emplace_back(std::make_unique<baseline::PermitLoopback>());
}

} // anonymous namespace

FwContext::FwContext
(
	uint32_t timeout
)
	: m_baseline(0)
	, m_activePolicy(Policy::None)
{
	auto engine = wfp::FilterEngine::StandardSession(timeout);

	//
	// Pass engine ownership to "session controller"
	//
	m_sessionController = std::make_unique<SessionController>(std::move(engine));

	if (false == applyBaseConfiguration())
	{
		THROW_ERROR("Failed to apply base configuration in BFE");
	}

	m_baseline = m_sessionController->checkpoint();
	m_activePolicy = Policy::None;
}

FwContext::FwContext
(
	uint32_t timeout,
	const WinFwSettings &settings,
	const std::optional<std::vector<WinFwAllowedEndpoint>>& allowedEndpoints
)
	: m_baseline(0)
	, m_activePolicy(Policy::None)
{
	auto engine = wfp::FilterEngine::StandardSession(timeout);

	//
	// Pass engine ownership to "session controller"
	//
	m_sessionController = std::make_unique<SessionController>(std::move(engine));

	uint32_t checkpoint = 0;

	if (false == applyBlockedBaseConfiguration(settings, allowedEndpoints, checkpoint))
	{
		THROW_ERROR("Failed to apply base configuration in BFE");
	}

	m_baseline = checkpoint;
	m_activePolicy = Policy::Blocked;
}

bool FwContext::applyPolicyConnecting
(
	const WinFwSettings& settings,
	const std::vector<WinFwAllowedEndpoint>& relays,

	const std::optional<std::wstring>& entryTunnelIfaceAlias,
	const WinFwAllowedTunnelTraffic& allowedEntryTunnelTraffic,

	const std::optional<std::wstring>& exitTunnelIfaceAlias,
	const WinFwAllowedTunnelTraffic& allowedExitTunnelTraffic,

	const std::optional<std::vector<WinFwAllowedEndpoint>>& allowedEndpoints,
	const std::vector<wfp::IpAddress>& nonTunnelDnsServers
)
{
	Ruleset ruleset;

	AppendNetBlockedRules(ruleset);
	AppendSettingsRules(ruleset, settings);
	AppendRelayRules(ruleset, relays);

	if (allowedEndpoints.has_value())
	{
		AppendAllowedEndpointRules(ruleset, allowedEndpoints.value());
	}

	if (!nonTunnelDnsServers.empty())
	{
		ruleset.emplace_back(std::make_unique<dns::PermitNonTunnel>(
			exitTunnelIfaceAlias, nonTunnelDnsServers
		));
	}

	//
	// Entry tunnel rules
	//
	if (entryTunnelIfaceAlias.has_value())
	{
		switch (allowedEntryTunnelTraffic.type)
		{
			case WinFwAllowedTunnelTrafficType::All:
			{
				ruleset.emplace_back(std::make_unique<baseline::PermitVpnTunnel>(
					baseline::PermitVpnTunnel::InterfaceType::Entry,
					*entryTunnelIfaceAlias,
					std::nullopt
				));
				ruleset.emplace_back(std::make_unique<baseline::PermitVpnTunnelService>(
					baseline::PermitVpnTunnel::InterfaceType::Entry,
					*entryTunnelIfaceAlias,
					std::nullopt
				));
				break;
			}
			case WinFwAllowedTunnelTrafficType::One:
			{
				auto onlyEndpoint = std::make_optional<baseline::PermitVpnTunnel::Endpoints>({
						baseline::PermitVpnTunnel::Endpoint{
						wfp::IpAddress(allowedEntryTunnelTraffic.endpoint1->ip),
						allowedEntryTunnelTraffic.endpoint1->port,
						allowedEntryTunnelTraffic.endpoint1->protocol
						},
						std::nullopt,
				});
				ruleset.emplace_back(std::make_unique<baseline::PermitVpnTunnel>(
					baseline::PermitVpnTunnel::InterfaceType::Entry,
					*entryTunnelIfaceAlias,
					onlyEndpoint
				));
				ruleset.emplace_back(std::make_unique<baseline::PermitVpnTunnelService>(
					baseline::PermitVpnTunnel::InterfaceType::Entry,
					*entryTunnelIfaceAlias,
					onlyEndpoint
				));
				break;
			}
			case WinFwAllowedTunnelTrafficType::Two:
			{
				auto endpoints = std::make_optional<baseline::PermitVpnTunnel::Endpoints>({
						baseline::PermitVpnTunnel::Endpoint{
						wfp::IpAddress(allowedEntryTunnelTraffic.endpoint1->ip),
						allowedEntryTunnelTraffic.endpoint1->port,
						allowedEntryTunnelTraffic.endpoint1->protocol
						},
						std::make_optional<baseline::PermitVpnTunnel::Endpoint>({
								wfp::IpAddress(allowedEntryTunnelTraffic.endpoint2->ip),
								allowedEntryTunnelTraffic.endpoint2->port,
								allowedEntryTunnelTraffic.endpoint2->protocol
								})
				});
				ruleset.emplace_back(std::make_unique<baseline::PermitVpnTunnel>(
							baseline::PermitVpnTunnel::InterfaceType::Entry,
							*entryTunnelIfaceAlias,
							endpoints
							));
				ruleset.emplace_back(std::make_unique<baseline::PermitVpnTunnelService>(
							baseline::PermitVpnTunnel::InterfaceType::Entry,
							*entryTunnelIfaceAlias,
							endpoints
							));
				break;
			}
			// For the "None" case, do nothing.
		}
	}

	//
	// Exit tunnel rules
	//
	if (exitTunnelIfaceAlias.has_value())
	{
		switch (allowedExitTunnelTraffic.type)
		{
		case WinFwAllowedTunnelTrafficType::All:
		{
			ruleset.emplace_back(std::make_unique<baseline::PermitVpnTunnel>(
				baseline::PermitVpnTunnel::InterfaceType::Exit,
				*exitTunnelIfaceAlias,
				std::nullopt
			));
			ruleset.emplace_back(std::make_unique<baseline::PermitVpnTunnelService>(
				baseline::PermitVpnTunnel::InterfaceType::Exit,
				*exitTunnelIfaceAlias,
				std::nullopt
			));
			break;
		}
		case WinFwAllowedTunnelTrafficType::One:
		{
			auto onlyEndpoint = std::make_optional<baseline::PermitVpnTunnel::Endpoints>({
					baseline::PermitVpnTunnel::Endpoint{
					wfp::IpAddress(allowedExitTunnelTraffic.endpoint1->ip),
					allowedExitTunnelTraffic.endpoint1->port,
					allowedExitTunnelTraffic.endpoint1->protocol
					},
					std::nullopt,
				});
			ruleset.emplace_back(std::make_unique<baseline::PermitVpnTunnel>(
				baseline::PermitVpnTunnel::InterfaceType::Exit,
				*exitTunnelIfaceAlias,
				onlyEndpoint
			));
			ruleset.emplace_back(std::make_unique<baseline::PermitVpnTunnelService>(
				baseline::PermitVpnTunnel::InterfaceType::Exit,
				*exitTunnelIfaceAlias,
				onlyEndpoint
			));
			break;
		}
		case WinFwAllowedTunnelTrafficType::Two:
		{
			auto endpoints = std::make_optional<baseline::PermitVpnTunnel::Endpoints>({
					baseline::PermitVpnTunnel::Endpoint{
					wfp::IpAddress(allowedExitTunnelTraffic.endpoint1->ip),
					allowedExitTunnelTraffic.endpoint1->port,
					allowedExitTunnelTraffic.endpoint1->protocol
					},
					std::make_optional<baseline::PermitVpnTunnel::Endpoint>({
							wfp::IpAddress(allowedExitTunnelTraffic.endpoint2->ip),
							allowedExitTunnelTraffic.endpoint2->port,
							allowedExitTunnelTraffic.endpoint2->protocol
							})
				});
			ruleset.emplace_back(std::make_unique<baseline::PermitVpnTunnel>(
				baseline::PermitVpnTunnel::InterfaceType::Exit,
				*exitTunnelIfaceAlias,
				endpoints
			));
			ruleset.emplace_back(std::make_unique<baseline::PermitVpnTunnelService>(
				baseline::PermitVpnTunnel::InterfaceType::Exit,
				*exitTunnelIfaceAlias,
				endpoints
			));
			break;
		}
		// For the "None" case, do nothing.
		}
	}

	const auto status = applyRuleset(ruleset);
	if (status)
	{
		m_activePolicy = Policy::Connecting;
	}

	return status;
}

bool FwContext::applyPolicyConnected
(
	const WinFwSettings& settings,
	const std::vector<WinFwAllowedEndpoint>& relays,
	const std::optional<std::wstring>&  entryTunnelIfaceAlias ,
	const std::optional<std::wstring>& exitTunnelIfaceAlias,
	const std::optional<std::vector<WinFwAllowedEndpoint>>& allowedEndpoints,
	const std::vector<wfp::IpAddress>& tunnelDnsServers,
	const std::vector<wfp::IpAddress>& nonTunnelDnsServers
)
{
	Ruleset ruleset;

	AppendNetBlockedRules(ruleset);
	AppendSettingsRules(ruleset, settings);
	AppendRelayRules(ruleset, relays);

	if (allowedEndpoints.has_value())
	{
		AppendAllowedEndpointRules(ruleset, allowedEndpoints.value());
	}

	if (exitTunnelIfaceAlias.has_value())
	{
		std::wstring exitTunnelIfaceAliasStr = exitTunnelIfaceAlias.value();

		if (!tunnelDnsServers.empty())
		{
			ruleset.emplace_back(std::make_unique<dns::PermitTunnel>(
				exitTunnelIfaceAliasStr, tunnelDnsServers
			));
		}

		if (!nonTunnelDnsServers.empty())
		{
			ruleset.emplace_back(std::make_unique<dns::PermitNonTunnel>(
				exitTunnelIfaceAliasStr, nonTunnelDnsServers
			));
		}

		ruleset.emplace_back(std::make_unique<baseline::PermitVpnTunnel>(
			baseline::PermitVpnTunnel::InterfaceType::Exit,
			exitTunnelIfaceAliasStr,
			std::nullopt
		));

		ruleset.emplace_back(std::make_unique<baseline::PermitVpnTunnelService>(
			baseline::PermitVpnTunnel::InterfaceType::Exit,
			exitTunnelIfaceAliasStr,
			std::nullopt
		));
	}

	if (entryTunnelIfaceAlias.has_value())
	{
		std::wstring entryTunnelIfaceAliasStr = entryTunnelIfaceAlias.value();

		
		ruleset.emplace_back(std::make_unique<baseline::PermitVpnTunnel>(
			baseline::PermitVpnTunnel::InterfaceType::Entry,
			entryTunnelIfaceAliasStr,
			std::nullopt
		));

		ruleset.emplace_back(std::make_unique<baseline::PermitVpnTunnelService>(
			baseline::PermitVpnTunnel::InterfaceType::Entry,
			entryTunnelIfaceAliasStr,
			std::nullopt
		));
	}


	const auto status = applyRuleset(ruleset);
	if (status)
	{
		m_activePolicy = Policy::Connected;
	}

	return status;
}

bool FwContext::applyPolicyBlocked(
	const WinFwSettings &settings, 
	const std::optional<std::vector<WinFwAllowedEndpoint>> &allowedEndpoints
)
{
	const auto status = applyRuleset(composePolicyBlocked(settings, allowedEndpoints));

	if (status)
	{
		m_activePolicy = Policy::Blocked;
	}

	return status;
}

bool FwContext::reset()
{
	const auto status = m_sessionController->executeTransaction([this](SessionController &controller, wfp::FilterEngine &)
	{
		return controller.revert(m_baseline), true;
	});

	if (status)
	{
		m_activePolicy = Policy::None;
	}

	return status;
}

FwContext::Policy FwContext::activePolicy() const
{
	return m_activePolicy;
}

FwContext::Ruleset FwContext::composePolicyBlocked
(
	const WinFwSettings &settings, 
	const std::optional<std::vector<WinFwAllowedEndpoint>> &allowedEndpoints
)
{
	Ruleset ruleset;

	AppendNetBlockedRules(ruleset);
	AppendSettingsRules(ruleset, settings);

	if (allowedEndpoints.has_value())
	{
		AppendAllowedEndpointRules(ruleset, allowedEndpoints.value());
	}

	return ruleset;
}

bool FwContext::applyBaseConfiguration()
{
	return m_sessionController->executeTransaction([this](SessionController &controller, wfp::FilterEngine &engine)
	{
		return applyCommonBaseConfiguration(controller, engine);
	});
}

bool FwContext::applyBlockedBaseConfiguration(const WinFwSettings &settings, const std::optional<std::vector<WinFwAllowedEndpoint>>& allowedEndpoints, uint32_t &checkpoint)
{
	return m_sessionController->executeTransaction([&](SessionController &controller, wfp::FilterEngine &engine)
	{
		if (false == applyCommonBaseConfiguration(controller, engine))
		{
			return false;
		}

		//
		// Record the current session state with only structural objects added.
		// If we snapshot at a later time we'd accidentally include the blocking policy rules
		// in the baseline checkpoint.
		//
		checkpoint = controller.peekCheckpoint();

		return applyRulesetDirectly(composePolicyBlocked(settings, allowedEndpoints), controller);
	});
}

bool FwContext::applyCommonBaseConfiguration(SessionController &controller, wfp::FilterEngine &engine)
{
	//
	// Since we're using a standard WFP session we can make no assumptions
	// about which objects are already installed since before.
	//
	ObjectPurger::GetRemoveAllFunctor()(engine);

	//
	// Install structural objects
	//
	return controller.addProvider(*MullvadObjects::Provider())
		&& controller.addSublayer(*MullvadObjects::SublayerBaseline())
		&& controller.addSublayer(*MullvadObjects::SublayerDns());
}

bool FwContext::applyRuleset(const Ruleset &ruleset)
{
	return m_sessionController->executeTransaction([&](SessionController &controller, wfp::FilterEngine &)
	{
		controller.revert(m_baseline);
		return applyRulesetDirectly(ruleset, controller);
	});
}

bool FwContext::applyRulesetDirectly(const Ruleset &ruleset, SessionController &controller)
{
	for (const auto &rule : ruleset)
	{
		if (false == rule->apply(controller))
		{
			return false;
		}
	}

	return true;
}
