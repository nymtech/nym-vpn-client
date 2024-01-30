#pragma once

#include <windows.h>
#include <functional>
#include <mutex>
#include <thread>

namespace common
{

class BurstGuard
{
public:

	using Callback = std::function<void()>;

	//
	// `burstDuration`
	// Requests to trigger the callback are clumped together if occuring within this timeframe.
	// The value is specified in milliseconds.
	//
	// `interferenceInterval`
	// For extended periods of bursts you may want to call through to the callback even
	// though the burst is still on-going.
	// The value is specified in milliseconds.
	// If specified as 0, it means interference is disabled.
	//
	BurstGuard(Callback callback, uint32_t burstDuration, uint32_t interferenceInterval = 0);
	~BurstGuard();

	BurstGuard(const BurstGuard &) = delete;
	BurstGuard &operator=(const BurstGuard &) = delete;
	BurstGuard(BurstGuard &&) = delete;
	BurstGuard &operator=(BurstGuard &&) = delete;

	void trigger();

private:

	Callback m_callback;
	uint32_t m_burstDuration;
	uint32_t m_interferenceInterval;

	std::mutex m_mutex;
	uint64_t m_timestampBurstStart;

	enum ThreadState
	{
		Sleeping,
		WaitingBurstEnd,
		Exiting,
	};

	ThreadState m_threadState;
	HANDLE m_stateEvent;
	HANDLE m_thread;

	static unsigned __stdcall Thread(void *context);
	void thread();
};

}
