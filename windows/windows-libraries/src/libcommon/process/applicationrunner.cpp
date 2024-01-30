#include "stdafx.h"
#include "applicationrunner.h"
#include "../error.h"
#include "../memory.h"
#include <memory>
#include <vector>
#include <windows.h>
#include <process.h>

using UniqueHandle = common::memory::UniqueHandle;

namespace common::process
{

ApplicationRunner::ApplicationRunner(const std::wstring &path, const std::wstring &args, DWORD creationFlags)
	: m_processId(0)
{
	auto cmdline = CreateCommandLine(path, args);

	createPipes();

	PROCESS_INFORMATION pi = { 0 };
	STARTUPINFOW si = { 0 };

	si.cb = sizeof(si);
	si.hStdError = *m_stdOutWrite.get();
	si.hStdOutput = *m_stdOutWrite.get();
	si.hStdInput = *m_stdInRead.get();
	si.dwFlags = STARTF_USESTDHANDLES;

	auto status = CreateProcessW(nullptr, const_cast<wchar_t *>(cmdline.c_str()),
		nullptr, nullptr, TRUE, creationFlags, nullptr, nullptr, &si, &pi);

	if (FALSE == status)
	{
		THROW_WINDOWS_ERROR(GetLastError(), "Create process");
	}

	CloseHandle(pi.hThread);

	m_processId = pi.dwProcessId;
	m_process.reset(new HANDLE(pi.hProcess));
}

//static
std::unique_ptr<ApplicationRunner> ApplicationRunner::StartDetached(
	const std::wstring &path, const std::wstring &args)
{
	return std::unique_ptr<ApplicationRunner>(new ApplicationRunner(path, args, DETACHED_PROCESS));
}

//static
std::unique_ptr<ApplicationRunner> ApplicationRunner::StartWithoutConsole(
	const std::wstring &path, const std::wstring &args)
{
	return std::unique_ptr<ApplicationRunner>(new ApplicationRunner(path, args, CREATE_NO_WINDOW));
}

bool ApplicationRunner::read(std::string &data, size_t maxChars, size_t timeout)
{
	//
	// There are no documented (approved) methods for issuing asynchronous operations
	// on anonymous pipes, so we have to fake it.
	//
	// Create a thread and have it issue synchronous read operations.
	//
	// TODO: At least reuse the thread :-(
	//

	UniqueHandle readCompletedEvent(new HANDLE(
		CreateEventW(nullptr, TRUE, FALSE, nullptr)
	));

	ReadThreadParameters parameters =
	{
		data,
		maxChars,
		*m_stdOutRead,
		*readCompletedEvent
	};

	UniqueHandle thread(new HANDLE(
		(HANDLE)_beginthreadex(nullptr, 0, &ApplicationRunner::ReadThread, &parameters, 0, nullptr)
	));

	if (WAIT_OBJECT_0 != WaitForSingleObject(*thread, static_cast<DWORD>(timeout)))
	{
		//
		// Timeout reached.
		//
		// Becauses the read may not yet have been registered on the other thread
		// we can't simply request to cancel the read and wait for the thread.
		//
		// We're forced to repeatedly cancel outstanding IO until the thread
		// acknowledges that it's done attempting to read.
		//

		do
		{
			CancelSynchronousIo(*thread);
		}
		while (WAIT_OBJECT_0 != WaitForSingleObject(*readCompletedEvent, 50));

		WaitForSingleObject(*thread, INFINITE);
	}

	DWORD exitCode;

	if (FALSE != GetExitCodeThread(*thread, &exitCode))
	{
		return static_cast<bool>(exitCode);
	}

	return false;
}

// static
unsigned __stdcall ApplicationRunner::ReadThread(void *p)
{
	auto parameters = reinterpret_cast<ReadThreadParameters *>(p);

	parameters->data.resize(parameters->maxChars);

	DWORD bytesRead;

	auto status = ReadFile(parameters->stdOutRead, &parameters->data[0],
		static_cast<DWORD>(parameters->maxChars), &bytesRead, nullptr);

	SetEvent(parameters->readCompletedEvent);

	if (FALSE == status || 0 == bytesRead)
	{
		return false;
	}

	parameters->data.resize(bytesRead);

	return true;
}

bool ApplicationRunner::write(const std::string &data)
{
	DWORD bytesWritten;

	auto status = WriteFile(*m_stdInWrite, data.c_str(), static_cast<DWORD>(data.size()), &bytesWritten, nullptr);

	return (FALSE != status && bytesWritten == data.size());
}

bool ApplicationRunner::join(DWORD &status, size_t timeout)
{
	if (WAIT_OBJECT_0 != WaitForSingleObject(*m_process, static_cast<DWORD>(timeout)))
	{
		return false;
	}

	if (FALSE == GetExitCodeProcess(*m_process, &status))
	{
		THROW_WINDOWS_ERROR(GetLastError(), "Read process exit code");
	}

	return true;
}

//static
std::wstring ApplicationRunner::CreateCommandLine(const std::wstring &path, const std::wstring &args)
{
	static const size_t SIZE_QUOTES_ETC = 4; // quote, quote, space, null

	auto cmdlineSize = path.size() + args.size() + SIZE_QUOTES_ETC;

	std::wstring cmdline;
	cmdline.reserve(cmdlineSize);

	cmdline.push_back(L'\"');
	cmdline.append(path);
	cmdline.append(L"\" ");
	cmdline.append(args);

	return cmdline;
}

void ApplicationRunner::createPipes()
{
	SECURITY_ATTRIBUTES sa;

	sa.nLength = sizeof(sa);
	sa.bInheritHandle = TRUE;
	sa.lpSecurityDescriptor = nullptr;

	m_stdInRead.reset(new HANDLE(INVALID_HANDLE_VALUE));
	m_stdInWrite.reset(new HANDLE(INVALID_HANDLE_VALUE));
	m_stdOutRead.reset(new HANDLE(INVALID_HANDLE_VALUE));
	m_stdOutWrite.reset(new HANDLE(INVALID_HANDLE_VALUE));

	if (FALSE == CreatePipe(m_stdInRead.get(), m_stdInWrite.get(), &sa, 0)
		|| FALSE == CreatePipe(m_stdOutRead.get(), m_stdOutWrite.get(), &sa, 0))
	{
		THROW_WINDOWS_ERROR(GetLastError(), "Create anonymous pipe");
	}

	if (FALSE == SetHandleInformation(*m_stdInWrite, HANDLE_FLAG_INHERIT, 0)
		|| FALSE == SetHandleInformation(*m_stdOutRead, HANDLE_FLAG_INHERIT, 0))
	{
		THROW_WINDOWS_ERROR(GetLastError(), "Disallow inheritance of pipe handle");
	}
}

}
