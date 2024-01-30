#pragma once

//
// This would be more suitably placed into `libwfp.h`, alas there is no such file
//
#pragma comment(lib, "fwpuclnt.lib")
#pragma comment(lib, "ws2_32.lib")
#pragma comment(lib, "iphlpapi.lib")

#include <fwpmtypes.h>
#include <windows.h>
#include <memory>

namespace wfp
{

class FilterEngine
{
	FilterEngine(bool dynamic, uint32_t *timeout);

public:

	// Create a session using a default timeout when waiting for the transaction lock.
	// The system default timeout on Windows 10 is 100 minutes.
	static std::unique_ptr<FilterEngine> DynamicSession();
	static std::unique_ptr<FilterEngine> StandardSession();

	// Create a session using a specific timeout when waiting for the transaction lock.
	// The timeout value is specified in milliseconds.
	// Specifying a timeout of INFINITE will cause an indefinite wait.
	// Zero is a reserved value in this context and means "system default timeout".
	static std::unique_ptr<FilterEngine> DynamicSession(uint32_t timeout);
	static std::unique_ptr<FilterEngine> StandardSession(uint32_t timeout);

	~FilterEngine();

	HANDLE session() const;

private:

	FilterEngine(const FilterEngine &) = delete;
	FilterEngine &operator=(const FilterEngine &) = delete;

	HANDLE m_session;
};

} // namespace wfp
