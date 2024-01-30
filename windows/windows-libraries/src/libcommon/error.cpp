#include "stdafx.h"
#include "error.h"
#include "string.h"
#include <exception>
#include <ios>
#include <iomanip>
#include <cstring>

namespace common::error {

WindowsException::WindowsException(const char *message, uint32_t errorCode)
	: std::runtime_error(message)
	, m_errorCode(errorCode)
{
}

std::string FormatWindowsError(DWORD errorCode)
{
	LPSTR buffer;

	auto status = FormatMessageA(FORMAT_MESSAGE_ALLOCATE_BUFFER | FORMAT_MESSAGE_FROM_SYSTEM | FORMAT_MESSAGE_IGNORE_INSERTS,
		nullptr, errorCode, 0, (LPSTR)&buffer, 0, nullptr);

	if (0 == status)
	{
		std::stringstream ss;

		ss << "System error 0x" << std::setw(8) << std::setfill('0') << std::hex << errorCode;

		return ss.str();
	}

	auto result = common::string::TrimRight(std::string(buffer));
	LocalFree(buffer);

	return result;
}

const char *IsolateFilename(const char *filepath)
{
	const auto slash = strrchr(filepath, '/');
	const auto backslash = strrchr(filepath, '\\');

	if (nullptr == slash && nullptr == backslash)
	{
		return filepath;
	}

	return max(slash, backslash) + 1;
}

[[noreturn]] void Throw(const char *operation, DWORD errorCode, const char *file, size_t line)
{
	std::stringstream ss;

	ss << operation << ": 0x" << std::setw(8) << std::setfill('0') << std::hex << errorCode
		<< std::setw(1) << std::dec
		<< ": " << common::error::FormatWindowsError(errorCode);

	Throw<WindowsException>(ss.str().c_str(), file, line, errorCode);
}

[[noreturn]] void Throw(const std::string &operation, DWORD errorCode, const char *file, size_t line)
{
	Throw(operation.c_str(), errorCode, file, line);
}

void UnwindException(const std::exception &err, std::shared_ptr<common::logging::ILogSink> logSink)
{
	logSink->error(err.what());

	try
	{
		std::rethrow_if_nested(err);
	}
	catch (const std::exception &innerErr)
	{
		UnwindException(innerErr, logSink);
	}
	catch (...)
	{
	}
}

}
