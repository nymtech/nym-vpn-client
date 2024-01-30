#include "stdafx.h"
#include "filterbuilder.h"
#include <libcommon/error.h>
#include <memory.h>

namespace wfp
{

FilterBuilder::FilterBuilder(BuilderValidation validation)
	: m_validation(validation)
{
	memset(&m_filter, 0, sizeof(m_filter));
}

FilterBuilder &FilterBuilder::key(const GUID &key)
{
	m_keyProvided = true;

	memcpy(&m_filter.filterKey, &key, sizeof(m_filter.filterKey));
	return *this;
}

FilterBuilder &FilterBuilder::name(const std::wstring &name)
{
	m_nameProvided = true;

	m_name = name;
	m_filter.displayData.name = const_cast<wchar_t *>(m_name.c_str());

	return *this;
}

FilterBuilder &FilterBuilder::description(const std::wstring &description)
{
	m_description = description;
	m_filter.displayData.description = const_cast<wchar_t *>(m_description.c_str());

	return *this;
}

FilterBuilder &FilterBuilder::persistent()
{
	m_filter.flags |= FWPM_FILTER_FLAG_PERSISTENT;
	return *this;
}

FilterBuilder &FilterBuilder::boottime()
{
	m_filter.flags |= FWPM_FILTER_FLAG_BOOTTIME;
	return *this;
}


FilterBuilder &FilterBuilder::disabled()
{
	m_filter.flags |= FWPM_FILTER_FLAG_DISABLED;
	return *this;
}

FilterBuilder &FilterBuilder::definitive()
{
	m_filter.flags |= FWPM_FILTER_FLAG_CLEAR_ACTION_RIGHT;
	return *this;
}

FilterBuilder &FilterBuilder::provider(const GUID &provider)
{
	m_providerProvided = true;

	memcpy(&m_providerKey, &provider, sizeof(m_providerKey));
	m_filter.providerKey = &m_providerKey;

	return *this;
}

FilterBuilder &FilterBuilder::layer(const GUID &layer)
{
	m_layerProvided = true;

	memcpy(&m_filter.layerKey, &layer, sizeof(m_filter.layerKey));
	return *this;
}

FilterBuilder &FilterBuilder::sublayer(const GUID &sublayer)
{
	m_sublayerProvided = true;

	memcpy(&m_filter.subLayerKey, &sublayer, sizeof(m_filter.subLayerKey));
	return *this;
}

FilterBuilder &FilterBuilder::weight(UINT64 weight)
{
	m_weightProvided = true;

	m_weight = weight;

	m_filter.weight.type = FWP_UINT64;
	m_filter.weight.uint64 = &m_weight;

	return *this;
}

FilterBuilder &FilterBuilder::weight(WeightClass cls)
{
	m_weightProvided = true;

	m_filter.weight.type = FWP_UINT8;
	m_filter.weight.uint8 = static_cast<UINT8>(cls);

	return *this;
}

FilterBuilder &FilterBuilder::block()
{
	m_verdictProvided = true;

	m_filter.action.type = FWP_ACTION_BLOCK;
	return *this;
}

FilterBuilder &FilterBuilder::permit()
{
	m_verdictProvided = true;

	m_filter.action.type = FWP_ACTION_PERMIT;
	return *this;
}

bool FilterBuilder::build(FilterSink sink)
{
	switch (m_validation)
	{
		case BuilderValidation::Extra:
		{
			if (false == m_keyProvided
				|| false == m_providerProvided
				|| false == m_sublayerProvided)
			{
				THROW_ERROR("Rejecting partially initialized sublayer");
			}

			// No break, fall through.
		}
		case BuilderValidation::OnlyCritical:
		{
			if (false == m_nameProvided
				|| false == m_layerProvided
				|| false == m_weightProvided
				|| false == m_verdictProvided)
			{
				THROW_ERROR("Rejecting partially initialized sublayer");
			}

			break;
		}
	}

	return sink(m_filter);
}

const GUID &FilterBuilder::id() const
{
	return m_filter.filterKey;
}

}
