#include "stdafx.h"
#include "objectmonitor.h"
#include <libcommon/error.h>

namespace wfp
{

ObjectMonitor::ObjectMonitor(std::shared_ptr<FilterEngine> engine)
	: m_engine(engine)
	, m_eventSubscription(nullptr)
{
}

ObjectMonitor::~ObjectMonitor()
{
	monitorEventsStop();
}

void ObjectMonitor::monitorEvents(std::function<void(const FWPM_NET_EVENT1&)> callback)
{
	m_eventCallback = callback;

	FWPM_NET_EVENT_SUBSCRIPTION0 spec = { 0 };

	auto status = FwpmNetEventSubscribe0(m_engine->session(), &spec, EventCallback, this, &m_eventSubscription);

	if (ERROR_SUCCESS != status)
	{
		THROW_WINDOWS_ERROR(status, "Register subscription for net events.");
	}
}

//static
void CALLBACK ObjectMonitor::EventCallback(void *context, const FWPM_NET_EVENT1 *event)
{
	reinterpret_cast<ObjectMonitor *>(context)->m_eventCallback(*event);
}

void ObjectMonitor::monitorEventsStop()
{
	FwpmNetEventUnsubscribe0(m_engine->session(), m_eventSubscription);
	m_eventSubscription = nullptr;
}

}
