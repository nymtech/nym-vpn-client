#include "stdafx.h"
#include "winfw.h"
#include "fwcontext.h"
#include "objectpurger.h"
#include "mullvadobjects.h"
#include "rules/persistent/blockall.h"
#include "libwfp/ipnetwork.h"
#include <windows.h>
#include <libcommon/error.h>
#include <libcommon/string.h>
#include <optional>

namespace
{

constexpr uint32_t DEINITIALIZE_TIMEOUT = 5000;

MullvadLogSink g_logSink = nullptr;
void *g_logSinkContext = nullptr;

FwContext *g_fwContext = nullptr;

WINFW_POLICY_STATUS
HandlePolicyException(const common::error::WindowsException &err)
{
	if (nullptr != g_logSink)
	{
		g_logSink(MULLVAD_LOG_LEVEL_ERROR, err.what(), g_logSinkContext);
	}

	if (FWP_E_TIMEOUT == err.errorCode())
	{
		// TODO: Detect software that may cause this
		return WINFW_POLICY_STATUS_LOCK_TIMEOUT;
	}

	return WINFW_POLICY_STATUS_GENERAL_FAILURE;
}

template<typename T>
std::optional<T> MakeOptional(T* object)
{
	if (nullptr == object)
	{
		return std::nullopt;
	}
	return std::make_optional(*object);
}

std::optional<std::wstring> MakeOptionalStr(const wchar_t* str)
{
	if (str == nullptr || *str == L'\0') 
	{
		return std::nullopt;
	}
	return std::wstring(str);
}

template<typename T>
std::optional<std::vector<T>> MakeOptionalVector(const T* const* items, size_t count)
{
	if (items == nullptr || count == 0)
	{
		return std::nullopt;
	}

	std::vector<T> result;
	result.reserve(count);

	for (size_t i = 0; i < count; ++i)
	{
		if (items[i] != nullptr)
		{
			result.emplace_back(*items[i]);
		}
	}

	return result.empty() ? std::nullopt : std::optional<std::vector<T>>(std::move(result));
}

template<typename T>
std::vector<T> MakeVector(const T* const* items, size_t count)
{
	if (items == nullptr || count == 0)
	{
		return {};
	}

	std::vector<T> result;
	result.reserve(count);

	for (size_t i = 0; i < count; ++i)
	{
		if (items[i] != nullptr)
		{
			result.emplace_back(*items[i]);
		}
	}

	return result;
}

std::vector<wfp::IpAddress> MakeIpAddressVector(const wchar_t* const* data, size_t num)
{
	if (data == nullptr || num == 0)
	{
		return {};
	}

	std::vector<wfp::IpAddress> result;
	result.reserve(num);

	for (size_t i = 0; i < num; ++i)
	{
		if (data[i] != nullptr)
		{
			result.emplace_back(data[i]);
		}
	}

	return result;
}

void LogDnsServers(const char* label, const std::vector<wfp::IpAddress>& dnsServers)
{
	if (nullptr == g_logSink)
	{
		return;
	}

	std::stringstream ss;
	ss << label << ": ";
	for (size_t i = 0; i < dnsServers.size(); i++)
	{
		if (i > 0)
		{
			ss << ", ";
		}
		ss << common::string::ToAnsi(dnsServers[i].toString());
	}
	g_logSink(MULLVAD_LOG_LEVEL_DEBUG, ss.str().c_str(), g_logSinkContext);
}

void LogAllowedEndpoints(const char *label, std::vector<WinFwAllowedEndpoint>& allowed_endpoints)
{
	if (nullptr == g_logSink)
	{
		return;
	}

	std::stringstream ss;
	ss << label << ": ";
	for (size_t i = 0; i < allowed_endpoints.size(); i++)
	{
		if (i > 0)
		{
			ss << ", ";
		}
		ss << common::string::ToAnsi(allowed_endpoints[i].endpoint.ip) << ":" << allowed_endpoints[i].endpoint.port << " ";

		switch (allowed_endpoints[i].endpoint.protocol) {
		case WinFwProtocol::Tcp:
			ss << "tcp";
			break;
		case WinFwProtocol::Udp:
			ss << "udp";
			break;
		default:
			ss << "unknown";
			break;
		}
	}
	g_logSink(MULLVAD_LOG_LEVEL_DEBUG, ss.str().c_str(), g_logSinkContext);
}

void LogInterface(const char* label, std::optional<std::wstring>& iface) 
{
	if (nullptr == g_logSink)
	{
		return;
	}

	std::stringstream ss;
	ss << label << ": ";

	if (iface.has_value()) {
		ss << common::string::ToAnsi(iface.value());
	}
	else {
		ss << "unset";
	}

	g_logSink(MULLVAD_LOG_LEVEL_DEBUG, ss.str().c_str(), g_logSinkContext);
}

} // anonymous namespace

WINFW_LINKAGE
bool
WINFW_API
WinFw_Initialize(
	uint32_t timeout,
	MullvadLogSink logSink,
	void *logSinkContext
)
{
	try
	{
		if (nullptr != g_fwContext)
		{
			//
			// This is an error.
			// The existing instance may have a different timeout etc.
			//
			THROW_ERROR("Cannot initialize WINFW twice");
		}

		// Convert seconds to milliseconds.
		uint32_t timeout_ms = timeout * 1000;

		g_logSink = logSink;
		g_logSinkContext = logSinkContext;

		g_fwContext = new FwContext(timeout_ms);
	}
	catch (std::exception &err)
	{
		if (nullptr != g_logSink)
		{
			g_logSink(MULLVAD_LOG_LEVEL_ERROR, err.what(), g_logSinkContext);
		}

		return false;
	}
	catch (...)
	{
		return false;
	}

	return true;
}

extern "C"
WINFW_LINKAGE
bool
WINFW_API
WinFw_InitializeBlocked(
	uint32_t timeout,
	const WinFwSettings *settings,
	const WinFwAllowedEndpoint *allowedEndpoints[],
	size_t numAllowedEndpoints,
	MullvadLogSink logSink,
	void *logSinkContext
)
{
	try
	{
		if (nullptr != g_fwContext)
		{
			//
			// This is an error.
			// The existing instance may have a different timeout etc.
			//
			THROW_ERROR("Cannot initialize WINFW twice");
		}

		if (nullptr == settings)
		{
			THROW_ERROR("Invalid argument: settings");
		}

		// Convert seconds to milliseconds.
		uint32_t timeout_ms = timeout * 1000;

		g_logSink = logSink;
		g_logSinkContext = logSinkContext;

		auto allowedEndpointVector = MakeOptionalVector(allowedEndpoints, numAllowedEndpoints);

		g_fwContext = new FwContext(timeout_ms, *settings, allowedEndpointVector);
	}
	catch (std::exception &err)
	{
		if (nullptr != g_logSink)
		{
			g_logSink(MULLVAD_LOG_LEVEL_ERROR, err.what(), g_logSinkContext);
		}

		return false;
	}
	catch (...)
	{
		return false;
	}

	return true;
}

WINFW_LINKAGE
bool
WINFW_API
WinFw_Deinitialize(WINFW_CLEANUP_POLICY cleanupPolicy)
{
	if (nullptr == g_fwContext)
	{
		return true;
	}

	const auto activePolicy = g_fwContext->activePolicy();

	//
	// Do not use FwContext::reset() here because it just
	// removes the current policy but leaves sublayers etc.
	//

	delete g_fwContext;
	g_fwContext = nullptr;

	//
	// Continue blocking if this is what the caller requested
	// and if the current policy is "(net) blocked".
	//

	if (WINFW_CLEANUP_POLICY_CONTINUE_BLOCKING == cleanupPolicy
		&& FwContext::Policy::Blocked == activePolicy)
	{
		try
		{
			auto engine = wfp::FilterEngine::StandardSession(DEINITIALIZE_TIMEOUT);
			auto sessionController = std::make_unique<SessionController>(std::move(engine));

			rules::persistent::BlockAll blockAll;

			return sessionController->executeTransaction([&](SessionController &controller, wfp::FilterEngine &engine)
			{
				ObjectPurger::GetRemoveNonPersistentFunctor()(engine);

				return controller.addProvider(*MullvadObjects::ProviderPersistent())
					&& controller.addSublayer(*MullvadObjects::SublayerPersistent())
					&& blockAll.apply(controller);
			});
		}
		catch (std::exception & err)
		{
			if (nullptr != g_logSink)
			{
				g_logSink(MULLVAD_LOG_LEVEL_ERROR, err.what(), g_logSinkContext);
			}
			return false;
		}
		catch (...)
		{
			return false;
		}
	}

	return WINFW_POLICY_STATUS_SUCCESS == WinFw_Reset();
}

WINFW_LINKAGE
WINFW_POLICY_STATUS
WINFW_API
WinFw_ApplyPolicyConnecting(
	const WinFwSettings* settings,
	const WinFwAllowedEndpoint* relays[],
	size_t numRelays,
	const wchar_t* entryTunnelIfaceAlias,
	const wchar_t* exitTunnelIfaceAlias,
	const WinFwAllowedEndpoint* allowedEndpoints[],
	size_t numAllowedEndpoints,
	const WinFwAllowedTunnelTraffic* allowedEntryTunnelTraffic,
	const WinFwAllowedTunnelTraffic* allowedExitTunnelTraffic,
	const wchar_t* nonTunnelDnsServers[],
	size_t numNonTunnelDnsServers
)
{
	if (nullptr == g_fwContext)
	{
		return WINFW_POLICY_STATUS_GENERAL_FAILURE;
	}

	try
	{
		if (nullptr == settings)
		{
			THROW_ERROR("Invalid argument: settings");
		}

		if (nullptr == allowedEntryTunnelTraffic)
		{
			THROW_ERROR("Invalid argument: allowedEntryTunnelTraffic");
		}

		if (nullptr == allowedExitTunnelTraffic)
		{
			THROW_ERROR("Invalid argument: allowedExitTunnelTraffic");
		}

		auto relayVector = MakeVector(relays, numRelays);
		auto entryTunnelIfaceAliasOptStr = MakeOptionalStr(entryTunnelIfaceAlias);
		auto exitTunnelIfaceAliasOptStr = MakeOptionalStr(exitTunnelIfaceAlias);
		auto allowedEndpointOptVector = MakeOptionalVector(allowedEndpoints, numAllowedEndpoints);
		auto nonTunnelDnsServerVector = MakeIpAddressVector(nonTunnelDnsServers, numNonTunnelDnsServers);

		LogAllowedEndpoints("Relays", relayVector);
		if (allowedEndpointOptVector.has_value()) {
			LogAllowedEndpoints("AllowedEndpoints", allowedEndpointOptVector.value());
		}
		LogInterface("entryTunnelIface", entryTunnelIfaceAliasOptStr);
		LogInterface("exitTunnelIface", exitTunnelIfaceAliasOptStr);
		LogDnsServers("Non-tunnel DNS servers", nonTunnelDnsServerVector);

		return g_fwContext->applyPolicyConnecting(
			*settings,
			relayVector,
			entryTunnelIfaceAliasOptStr,
			*allowedEntryTunnelTraffic,
			exitTunnelIfaceAliasOptStr,
			*allowedExitTunnelTraffic,
			allowedEndpointOptVector,
			nonTunnelDnsServerVector
		) ? WINFW_POLICY_STATUS_SUCCESS : WINFW_POLICY_STATUS_GENERAL_FAILURE;
	}
	catch (common::error::WindowsException &err)
	{
		return HandlePolicyException(err);
	}
	catch (std::exception &err)
	{
		if (nullptr != g_logSink)
		{
			g_logSink(MULLVAD_LOG_LEVEL_ERROR, err.what(), g_logSinkContext);
		}

		return WINFW_POLICY_STATUS_GENERAL_FAILURE;
	}
	catch (...)
	{
		return WINFW_POLICY_STATUS_GENERAL_FAILURE;
	}
}

WINFW_LINKAGE
WINFW_POLICY_STATUS
WINFW_API
WinFw_ApplyPolicyConnected(
	const WinFwSettings* settings,
	const WinFwAllowedEndpoint* relays[],
	size_t numRelays,
	const wchar_t* entryTunnelIfaceAlias,
	const wchar_t* exitTunnelIfaceAlias,
	const wchar_t* tunnelDnsServers[],
	size_t numTunnelDnsServers,
	const wchar_t* nonTunnelDnsServers[],
	size_t numNonTunnelDnsServers,
	const WinFwAllowedEndpoint* allowedEndpoints[],
	size_t numAllowedEndpoints
)
{
	if (nullptr == g_fwContext)
	{
		return WINFW_POLICY_STATUS_GENERAL_FAILURE;
	}

	try
	{
		if (nullptr == settings)
		{
			THROW_ERROR("Invalid argument: settings");
		}

		if (nullptr == relays)
		{
			THROW_ERROR("Invalid argument: relays");
		}

		if (nullptr == tunnelDnsServers)
		{
			THROW_ERROR("Invalid argument: tunnelDnsServers");
		}

		if (nullptr == nonTunnelDnsServers)
		{
			THROW_ERROR("Invalid argument: nonTunnelDnsServers");
		}

		auto relayVector = MakeVector(relays, numRelays);
		auto entryTunnelIfaceAliasOptStr = MakeOptionalStr(entryTunnelIfaceAlias);
		auto exitTunnelIfaceAliasOptStr = MakeOptionalStr(exitTunnelIfaceAlias);
		auto allowedEndpointOptVector = MakeOptionalVector(allowedEndpoints, numAllowedEndpoints);
		auto nonTunnelDnsServerVector = MakeIpAddressVector(nonTunnelDnsServers, numNonTunnelDnsServers);
		auto tunnelDnsServersVector = MakeIpAddressVector(tunnelDnsServers, numTunnelDnsServers);

		LogAllowedEndpoints("Relays", relayVector);
		if (allowedEndpointOptVector.has_value()) {
			LogAllowedEndpoints("Allowed endpoints", allowedEndpointOptVector.value());
		}
		LogInterface("Entry tunnel interface", entryTunnelIfaceAliasOptStr);
		LogInterface("Exit tunnel interface", exitTunnelIfaceAliasOptStr);
		LogDnsServers("Non-tunnel DNS servers", nonTunnelDnsServerVector);
		LogDnsServers("Tunnel DNS servers", tunnelDnsServersVector);

		return g_fwContext->applyPolicyConnected(
			*settings,
			relayVector,
			entryTunnelIfaceAliasOptStr,
			exitTunnelIfaceAliasOptStr,
			allowedEndpointOptVector,
			tunnelDnsServersVector,
			nonTunnelDnsServerVector
		) ? WINFW_POLICY_STATUS_SUCCESS : WINFW_POLICY_STATUS_GENERAL_FAILURE;
	}
	catch (common::error::WindowsException &err)
	{
		return HandlePolicyException(err);
	}
	catch (std::exception &err)
	{
		if (nullptr != g_logSink)
		{
			g_logSink(MULLVAD_LOG_LEVEL_ERROR, err.what(), g_logSinkContext);
		}

		return WINFW_POLICY_STATUS_GENERAL_FAILURE;
	}
	catch (...)
	{
		return WINFW_POLICY_STATUS_GENERAL_FAILURE;
	}
}

WINFW_LINKAGE
WINFW_POLICY_STATUS
WINFW_API
WinFw_ApplyPolicyBlocked(
	const WinFwSettings* settings,
	const WinFwAllowedEndpoint* allowedEndpoints[],
	size_t numAllowedEndpoints
)
{
	if (nullptr == g_fwContext)
	{
		return WINFW_POLICY_STATUS_GENERAL_FAILURE;
	}

	try
	{
		if (nullptr == settings)
		{
			THROW_ERROR("Invalid argument: settings");
		}

		auto allowedEndpointVector = MakeOptionalVector(allowedEndpoints, numAllowedEndpoints);

		return g_fwContext->applyPolicyBlocked(*settings, allowedEndpointVector)
			? WINFW_POLICY_STATUS_SUCCESS
			: WINFW_POLICY_STATUS_GENERAL_FAILURE;
	}
	catch (common::error::WindowsException &err)
	{
		return HandlePolicyException(err);
	}
	catch (std::exception &err)
	{
		if (nullptr != g_logSink)
		{
			g_logSink(MULLVAD_LOG_LEVEL_ERROR, err.what(), g_logSinkContext);
		}

		return WINFW_POLICY_STATUS_GENERAL_FAILURE;
	}
	catch (...)
	{
		return WINFW_POLICY_STATUS_GENERAL_FAILURE;
	}
}

WINFW_LINKAGE
WINFW_POLICY_STATUS
WINFW_API
WinFw_Reset()
{
	try
	{
		if (nullptr == g_fwContext)
		{
			return ObjectPurger::Execute(ObjectPurger::GetRemoveAllFunctor())
				? WINFW_POLICY_STATUS_SUCCESS
				: WINFW_POLICY_STATUS_GENERAL_FAILURE;
		}

		return g_fwContext->reset()
			? WINFW_POLICY_STATUS_SUCCESS
			: WINFW_POLICY_STATUS_GENERAL_FAILURE;
	}
	catch (common::error::WindowsException &err)
	{
		return HandlePolicyException(err);
	}
	catch (std::exception &err)
	{
		if (nullptr != g_logSink)
		{
			g_logSink(MULLVAD_LOG_LEVEL_ERROR, err.what(), g_logSinkContext);
		}

		return WINFW_POLICY_STATUS_GENERAL_FAILURE;
	}
	catch (...)
	{
		return WINFW_POLICY_STATUS_GENERAL_FAILURE;
	}
}
