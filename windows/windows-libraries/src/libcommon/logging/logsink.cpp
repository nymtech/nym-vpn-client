#include "stdafx.h"
#include "logsink.h"

namespace common::logging
{

using LockType = std::scoped_lock<std::mutex>;

LogSink::LogSink(LogTarget target)
	: m_target(target)
{
}

void LogSink::setTarget(LogTarget target)
{
	LockType lock(m_mutex);
	m_target = target;
}

void LogSink::error(const char *msg)
{
	forward(LogLevel::Error, msg);
}

void LogSink::warning(const char *msg)
{
	forward(LogLevel::Warning, msg);
}

void LogSink::info(const char *msg)
{
	forward(LogLevel::Info, msg);
}

void LogSink::debug(const char* msg)
{
	forward(LogLevel::Debug, msg);
}

void LogSink::trace(const char *msg)
{
	forward(LogLevel::Trace, msg);
}

void LogSink::forward(LogLevel level, const char *msg)
{
	LockType lock(m_mutex);

	if (m_target)
	{
		m_target(level, msg);
	}
}

}
