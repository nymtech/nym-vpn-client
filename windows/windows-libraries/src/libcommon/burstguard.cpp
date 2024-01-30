#include "stdafx.h"
#include "burstguard.h"
#include <process.h>
#include <libcommon/error.h>

namespace common
{

BurstGuard::BurstGuard(Callback callback, uint32_t burstDuration, uint32_t interferenceInterval)
	: m_callback(callback)
	, m_burstDuration(burstDuration)
	, m_interferenceInterval(interferenceInterval)
	, m_timestampBurstStart(0)
	, m_threadState(ThreadState::Sleeping)
{
	m_stateEvent = CreateEventW(nullptr, TRUE, FALSE, nullptr);

	if (nullptr == m_stateEvent)
	{
		THROW_WINDOWS_ERROR(GetLastError(), "Create BurstGuard thread state event object");
	}

	const auto t = _beginthreadex(nullptr, 0, BurstGuard::Thread, this, 0, nullptr);

	if (0 == t)
	{
		CloseHandle(m_stateEvent);
		THROW_ERROR("Failed to create BurstGuard thread");
	}

	m_thread = reinterpret_cast<HANDLE>(t);
}

BurstGuard::~BurstGuard()
{
	{
		std::scoped_lock<std::mutex> lock(m_mutex);

		m_threadState = ThreadState::Exiting;
		SetEvent(m_stateEvent);
	}

	WaitForSingleObject(m_thread, INFINITE);

	CloseHandle(m_thread);
	CloseHandle(m_stateEvent);
}

void BurstGuard::trigger()
{
	std::scoped_lock<std::mutex> lock(m_mutex);

	const uint64_t timestampNow = static_cast<uint64_t>(GetTickCount64());

	if (0 == m_timestampBurstStart)
	{
		m_timestampBurstStart = timestampNow;
	}
	else if (0 != m_interferenceInterval
		&& timestampNow - m_timestampBurstStart >= m_interferenceInterval)
	{
		m_callback();
		m_timestampBurstStart = 0;

		m_threadState = ThreadState::Sleeping;
		SetEvent(m_stateEvent);

		return;
	}

	//
	// Reset timer.
	//

	m_threadState = ThreadState::WaitingBurstEnd;
	SetEvent(m_stateEvent);
}

//static
unsigned __stdcall BurstGuard::Thread(void *context)
{
	reinterpret_cast<BurstGuard *>(context)->thread();
	return 0;
}

void BurstGuard::thread()
{
	for (;;)
	{
		WaitForSingleObject(m_stateEvent, INFINITE);

ack_and_process:

		ThreadState state;

		{
			std::scoped_lock<std::mutex> lock(m_mutex);

			ResetEvent(m_stateEvent);
			state = m_threadState;
		}

		if (ThreadState::Sleeping == state)
		{
			continue;
		}

		if (ThreadState::Exiting == state)
		{
			break;
		}

		//
		// Wait for burst to end.
		//

		if (WAIT_TIMEOUT != WaitForSingleObject(m_stateEvent, m_burstDuration))
		{
			goto ack_and_process;
		}

		//
		// Burst has ended.
		//

		{
			std::scoped_lock<std::mutex> lock(m_mutex);

			if (0 != m_timestampBurstStart)
			{
				m_callback();
				m_timestampBurstStart = 0;
			}
		}
	}
}

}
