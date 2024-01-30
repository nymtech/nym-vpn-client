#include "stdafx.h"
#include "sublayerbuilder.h"
#include <memory.h>

namespace wfp
{

SublayerBuilder::SublayerBuilder(BuilderValidation validation)
	: m_validation(validation)
{
	memset(&m_sublayer, 0, sizeof(m_sublayer));
}

SublayerBuilder &SublayerBuilder::key(const GUID &key)
{
	m_keyProvided = true;

	memcpy(&m_sublayer.subLayerKey, &key, sizeof(m_sublayer.subLayerKey));
	return *this;
}

SublayerBuilder &SublayerBuilder::name(const std::wstring &name)
{
	m_nameProvided = true;

	m_name = name;
	m_sublayer.displayData.name = const_cast<wchar_t *>(m_name.c_str());

	return *this;
}

SublayerBuilder &SublayerBuilder::description(const std::wstring &description)
{
	m_description = description;
	m_sublayer.displayData.description = const_cast<wchar_t *>(m_description.c_str());

	return *this;
}

SublayerBuilder &SublayerBuilder::persistent()
{
	m_sublayer.flags |= FWPM_SUBLAYER_FLAG_PERSISTENT;
	return *this;
}

SublayerBuilder &SublayerBuilder::provider(const GUID &provider)
{
	m_providerProvided = true;

	memcpy(&m_providerKey, &provider, sizeof(m_providerKey));
	m_sublayer.providerKey = &m_providerKey;

	return *this;
}

SublayerBuilder &SublayerBuilder::data(const uint8_t *data, size_t size)
{
	m_data = std::make_unique<uint8_t[]>(size);
	m_dataSize = size;

	memcpy(m_data.get(), data, size);

	m_sublayer.providerData.data = m_data.get();
	m_sublayer.providerData.size = static_cast<UINT32>(m_dataSize);

	return *this;
}

SublayerBuilder &SublayerBuilder::weight(UINT16 weight)
{
	m_weightProvided = true;

	m_sublayer.weight = weight;
	return *this;
}

bool SublayerBuilder::build(SublayerSink sink)
{
	switch (m_validation)
	{
		case BuilderValidation::Extra:
		{
			if (false == m_keyProvided
				|| false == m_providerProvided)
			{
				THROW_ERROR("Rejecting partially initialized sublayer");
			}

			// No break, fall through.
		}
		case BuilderValidation::OnlyCritical:
		{
			if (false == m_nameProvided
				|| false == m_weightProvided)
			{
				THROW_ERROR("Rejecting partially initialized sublayer");
			}

			break;
		}
	}

	return sink(m_sublayer);
}

const GUID &SublayerBuilder::id() const
{
	return m_sublayer.subLayerKey;
}

}
