#pragma once

#include "ifiltercondition.h"
#include <libcommon/buffer.h>
#include <memory>

namespace wfp::conditions {

class ConditionProtocol : public IFilterCondition
{
	enum class Protocol
	{
		Tcp,
		Udp,
		Icmp,
		IcmpV6,
		Ip,
		IpV6,
		Raw
	};

	ConditionProtocol(Protocol protocol);

public:

	static std::unique_ptr<ConditionProtocol> Tcp();
	static std::unique_ptr<ConditionProtocol> Udp();
	static std::unique_ptr<ConditionProtocol> Icmp();
	static std::unique_ptr<ConditionProtocol> IcmpV6();
	static std::unique_ptr<ConditionProtocol> Ip();
	static std::unique_ptr<ConditionProtocol> IpV6();
	static std::unique_ptr<ConditionProtocol> Raw();

	std::wstring toString() const override;
	const GUID &identifier() const override;
	const FWPM_FILTER_CONDITION0 &condition() const override;

private:

	Protocol m_protocol;
	common::Buffer m_assembled;

	static uint8_t TranslateProtocol(Protocol protocol);
};

}
