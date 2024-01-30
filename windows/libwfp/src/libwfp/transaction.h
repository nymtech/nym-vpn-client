#pragma once

#include "filterengine.h"
#include <functional>
#include <memory>

namespace wfp
{

class Transaction
{
public:

	static bool Execute(FilterEngine &engine, std::function<bool()> operation);
	static bool ExecuteReadOnly(FilterEngine &engine, std::function<bool()> operation);

private:

	static bool Execute(FilterEngine &engine, std::function<bool()> operation, bool readWrite);
};

} // namespace wfp
