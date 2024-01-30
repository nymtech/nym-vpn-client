#pragma once

#include "iidentifiable.h"
#include "buildervalidation.h"
#include <guiddef.h>
#include <fwpmu.h>
#include <windows.h>
#include <functional>
#include <memory>
#include <string>

namespace wfp
{

typedef std::function<bool(FWPM_SUBLAYER0 &)> SublayerSink;

class SublayerBuilder : public IIdentifiable
{
public:

	SublayerBuilder(BuilderValidation validation = BuilderValidation::Extra);

	SublayerBuilder(const SublayerBuilder&) = delete;
	SublayerBuilder &operator=(const SublayerBuilder&) = delete;

	SublayerBuilder &key(const GUID &key);
	SublayerBuilder &name(const std::wstring &name);
	SublayerBuilder &description(const std::wstring &description);

	SublayerBuilder &persistent();
	SublayerBuilder &provider(const GUID &provider);
	SublayerBuilder &data(const uint8_t *data, size_t size);
	SublayerBuilder &weight(UINT16 weight);

	bool build(SublayerSink sink);

	const GUID &IIdentifiable::id() const override;

private:

	FWPM_SUBLAYER0 m_sublayer;

	std::wstring m_name;
	std::wstring m_description;
	GUID m_providerKey;
	std::unique_ptr<uint8_t[]> m_data;
	size_t m_dataSize;

	BuilderValidation m_validation;

	bool m_keyProvided = false;
	bool m_nameProvided = false;
	bool m_providerProvided = false;
	bool m_weightProvided = false;
};

}

