#pragma once

#include "ifiltercondition.h"
#include <libcommon/buffer.h>
#include <memory>

namespace wfp::conditions {

class ConditionDirection : public IFilterCondition
{
	enum class Direction
	{
		Inbound,
		Outbound
	};

	ConditionDirection(Direction direction);

public:

	static std::unique_ptr<ConditionDirection> Inbound();
	static std::unique_ptr<ConditionDirection> Outbound();

	std::wstring toString() const override;
	const GUID &identifier() const override;
	const FWPM_FILTER_CONDITION0 &condition() const override;

private:

	Direction m_direction;
	common::Buffer m_assembled;
};

}
