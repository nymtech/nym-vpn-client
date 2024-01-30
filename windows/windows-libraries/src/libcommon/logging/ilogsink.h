#pragma once

namespace common::logging
{

struct ILogSink
{
	virtual ~ILogSink() = 0
	{
	}

	virtual void error(const char *msg) = 0;
	virtual void warning(const char *msg) = 0;
	virtual void info(const char *msg) = 0;
	virtual void debug(const char* msg) = 0;
	virtual void trace(const char *msg) = 0;
};

}
