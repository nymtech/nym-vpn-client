#include "stdafx.h"
#include "transaction.h"
#include <libcommon/error.h>
#include <guiddef.h>
#include <fwpmu.h>

namespace wfp
{

//static
bool Transaction::Execute(FilterEngine &engine, std::function<bool()> operation)
{
	return Execute(engine, operation, true);
}

//static
bool Transaction::ExecuteReadOnly(FilterEngine &engine, std::function<bool()> operation)
{
	return Execute(engine, operation, false);
}

//static
bool Transaction::Execute(FilterEngine &engine, std::function<bool()> operation, bool readWrite)
{
	auto status = FwpmTransactionBegin0(engine.session(), (readWrite ? 0 : FWPM_TXN_READ_ONLY));

	if (ERROR_SUCCESS != status)
	{
		THROW_WINDOWS_ERROR(status, "Initiate WFP transaction");
	}

	bool operationResult = false;

	try
	{
		operationResult = operation();
	}
	catch (...)
	{
		// Attemp to abort and throw with inner exception if it fails
		status = FwpmTransactionAbort0(engine.session());

		if (ERROR_SUCCESS != status)
		{
			THROW_WINDOWS_ERROR(status, "Abort WFP transaction");
		}

		// Successfully aborted so rethrow original exception
		throw;
	}

	if (operationResult)
	{
		status = FwpmTransactionCommit0(engine.session());

		if (ERROR_SUCCESS != status)
		{
			THROW_WINDOWS_ERROR(status, "Commit WFP transaction");
		}
	}
	else
	{
		status = FwpmTransactionAbort0(engine.session());

		if (ERROR_SUCCESS != status)
		{
			THROW_WINDOWS_ERROR(status, "Abort WFP transaction");
		}
	}

	return operationResult;
}

} // namespace wfp
