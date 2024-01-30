#pragma once

#include "iidentifiable.h"
#include "buildervalidation.h"
#include <guiddef.h>
#include <fwpmu.h>
#include <functional>
#include <string>
#include <memory>

namespace wfp
{

typedef std::function<bool(FWPM_PROVIDER0 &)> ProviderSink;

class ProviderBuilder : public IIdentifiable
{
public:

	ProviderBuilder(BuilderValidation validation = BuilderValidation::Extra);

	ProviderBuilder(const ProviderBuilder&) = delete;
	ProviderBuilder &operator=(const ProviderBuilder&) = delete;

	ProviderBuilder &key(const GUID &key);
	ProviderBuilder &name(const std::wstring &name);
	ProviderBuilder &description(const std::wstring &description);

	ProviderBuilder &persistent();

	// Optional data to provide additional context.
	ProviderBuilder &data(const uint8_t *data, size_t size);

	// Optional name of Windows service hosting the provider.
	// This allows the BFE to detect that a provider has been disabled.
	ProviderBuilder &serviceName(const std::wstring &serviceName);

	bool build(ProviderSink sink);

	const GUID &IIdentifiable::id() const override;

private:

	FWPM_PROVIDER0 m_provider;

	std::wstring m_name;
	std::wstring m_description;
	std::unique_ptr<uint8_t[]> m_data;
	size_t m_dataSize;
	std::wstring m_serviceName;

	BuilderValidation m_validation;

	bool m_keyProvided = false;
	bool m_nameProvided = false;
};

}
