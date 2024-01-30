#include "stdafx.h"
#include "providerbuilder.h"
#include <memory.h>

namespace wfp
{

ProviderBuilder::ProviderBuilder(BuilderValidation validation)
	: m_validation(validation)
{
	memset(&m_provider, 0, sizeof(m_provider));
}

ProviderBuilder &ProviderBuilder::key(const GUID &key)
{
	m_keyProvided = true;

	memcpy(&m_provider.providerKey, &key, sizeof(m_provider.providerKey));
	return *this;
}

ProviderBuilder &ProviderBuilder::name(const std::wstring &name)
{
	m_nameProvided = true;

	m_name = name;
	m_provider.displayData.name = const_cast<wchar_t *>(m_name.c_str());

	return *this;
}

ProviderBuilder &ProviderBuilder::description(const std::wstring &description)
{
	m_description = description;
	m_provider.displayData.description = const_cast<wchar_t *>(m_description.c_str());

	return *this;
}

ProviderBuilder &ProviderBuilder::persistent()
{
	m_provider.flags |= FWPM_PROVIDER_FLAG_PERSISTENT;
	return *this;
}

ProviderBuilder &ProviderBuilder::data(const uint8_t *data, size_t size)
{
	m_data = std::make_unique<uint8_t[]>(size);
	m_dataSize = size;

	memcpy(m_data.get(), data, size);

	m_provider.providerData.data = m_data.get();
	m_provider.providerData.size = static_cast<UINT32>(m_dataSize);

	return *this;
}

ProviderBuilder &ProviderBuilder::serviceName(const std::wstring &serviceName)
{
	m_serviceName = serviceName;
	m_provider.serviceName = const_cast<wchar_t *>(m_serviceName.c_str());

	return *this;
}

bool ProviderBuilder::build(ProviderSink sink)
{
	switch (m_validation)
	{
		case BuilderValidation::Extra:
		{
			if (false == m_keyProvided)
			{
				THROW_ERROR("Rejecting partially initialized sublayer");
			}

			// No break, fall through.
		}
		case BuilderValidation::OnlyCritical:
		{
			if (false == m_nameProvided)
			{
				THROW_ERROR("Rejecting partially initialized sublayer");
			}

			break;
		}
	}

	return sink(m_provider);
}

const GUID &ProviderBuilder::id() const
{
	return m_provider.providerKey;
}

}
