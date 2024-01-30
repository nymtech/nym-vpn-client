#pragma once

#include <libcommon/process/applicationrunner.h>
#include <memory>
#include <string>

class Ncat
{
public:

	enum ReturnCode
	{
		Ok = 0,
		Error = 1
	};

	Ncat(const std::wstring &args);

	bool write(const std::string &data);
	std::string output();
	DWORD returnCode();

	static std::wstring Path();

private:

	std::unique_ptr<common::process::ApplicationRunner> m_nc;
};
