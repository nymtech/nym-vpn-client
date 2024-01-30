#pragma once

#include "filterengine.h"
#include <windows.h>
#include <fwpmu.h>
#include <functional>
#include <memory>

namespace wfp
{

class ObjectMonitor
{
public:

	ObjectMonitor(std::shared_ptr<FilterEngine> engine);
	~ObjectMonitor();

	void monitorEvents(std::function<void(const FWPM_NET_EVENT1&)> callback);
	void monitorEventsStop();
	
private:

	std::shared_ptr<FilterEngine> m_engine;

	std::function<void(const FWPM_NET_EVENT1&)> m_eventCallback;
	HANDLE m_eventSubscription;

	static void CALLBACK EventCallback(void *context, const FWPM_NET_EVENT1 *event);
};

}
