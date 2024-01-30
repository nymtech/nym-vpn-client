#pragma once

#include "filterengine.h"
#include <functional>
#include <memory>
#include <fwpmtypes.h>
#include <windows.h>

namespace wfp
{

class ObjectEnumerator
{
	ObjectEnumerator();

public:

	static bool Sessions(FilterEngine &engine, std::function<bool(const FWPM_SESSION0&)> callback);
	static bool Providers(FilterEngine &engine, std::function<bool(const FWPM_PROVIDER0&)> callback);
	static bool Events(FilterEngine &engine, std::function<bool(const FWPM_NET_EVENT0&)> callback);
	static bool Filters(FilterEngine &engine, std::function<bool(const FWPM_FILTER0&)> callback);
	static bool Layers(FilterEngine &engine, std::function<bool(const FWPM_LAYER0&)> callback);
	static bool ProviderContexts(FilterEngine &engine, std::function<bool(const FWPM_PROVIDER_CONTEXT0&)> callback);
	static bool Sublayers(FilterEngine &engine, std::function<bool(const FWPM_SUBLAYER0&)> callback);
};

} // namespace wfp
