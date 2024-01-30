#include "pch.h"
#include "libcommon/string.h"
#include "CppUnitTest.h"
#include <algorithm>

using namespace Microsoft::VisualStudio::CppUnitTestFramework;

namespace testlibcommon
{

TEST_CLASS(TestLibCommonString)
{
public:

	TEST_METHOD(JoinNothing)
	{
		Assert::AreEqual(L"", common::string::Join(std::vector<std::wstring>()).c_str());
	}

	TEST_METHOD(JoinSingleElement)
	{
		Assert::AreEqual(L"foobar", common::string::Join(std::vector<std::wstring>() = { L"foobar" }).c_str());
	}

	TEST_METHOD(JoinMultipleElements)
	{
		Assert::AreEqual(L"foobar, foobear", common::string::Join(std::vector<std::wstring>() =
			{ L"foobar", L"foobear" }).c_str());
	}

	TEST_METHOD(FormatFlagsNoMatch)
	{
		std::vector<std::pair<UINT32, std::wstring> > definitions =
		{
			std::make_pair(0x01, L"FLAG_ONE")
		};

		Assert::AreEqual(L"[...]", common::string::FormatFlags(definitions, (UINT32)0x02).c_str());
	}

	TEST_METHOD(FormatFlagsOneMatchOneFlag)
	{
		std::vector<std::pair<UINT32, std::wstring> > definitions =
		{
			std::make_pair(0x01, L"FLAG_ONE")
		};

		Assert::AreEqual(L"FLAG_ONE", common::string::FormatFlags(definitions, (UINT32)0x01).c_str());
	}

	TEST_METHOD(FormatFlagsTwoMatchesTwoFlags)
	{
		std::vector<std::pair<UINT32, std::wstring> > definitions =
		{
			std::make_pair(0x01, L"FLAG_ONE"),
			std::make_pair(0x02, L"FLAG_TWO")
		};

		Assert::AreEqual(L"FLAG_ONE, FLAG_TWO", common::string::FormatFlags(definitions, (UINT32)0x03).c_str());
	}

	TEST_METHOD(FormatFlagsOneMatchSeveralFlags)
	{
		std::vector<std::pair<UINT32, std::wstring> > definitions =
		{
			std::make_pair(0x01, L"FLAG_ONE")
		};

		Assert::AreEqual(L"FLAG_ONE, [...]", common::string::FormatFlags(definitions, (UINT32)0x03).c_str());
	}

	TEST_METHOD(FormatFlagsOneMatchSeveralFlagsManyDefinitions)
	{
		std::vector<std::pair<UINT32, std::wstring> > definitions =
		{
			std::make_pair(0x01, L"FLAG_ONE"),
			std::make_pair(0x04, L"FLAG_THREE"),
			std::make_pair(0x08, L"FLAG_FOUR")
		};

		Assert::AreEqual(L"FLAG_ONE, [...]", common::string::FormatFlags(definitions, (UINT32)0x03).c_str());
	}

	TEST_METHOD(FormatIpV4)
	{
		Assert::AreEqual(L"127.0.0.1", common::string::FormatIpv4(0x7f000001).c_str());
	}

	TEST_METHOD(FormatIpV6)
	{
		UINT8 ip[] =
		{
			0x10, 0x00, 0x20, 0x00, 0x30, 0x00, 0x40, 0x00,
			0x50, 0x00, 0x60, 0x00, 0x70, 0x00, 0x80, 0x00
		};

		Assert::AreEqual(L"1000:2000:3000:4000:5000:6000:7000:8000", common::string::FormatIpv6(ip).c_str());
	}

};

}
