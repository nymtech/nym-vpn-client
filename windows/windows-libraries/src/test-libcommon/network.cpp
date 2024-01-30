#include "pch.h"
#include "libcommon/network.h"
#include "CppUnitTest.h"

using namespace Microsoft::VisualStudio::CppUnitTestFramework;

namespace testlibcommon
{

TEST_CLASS(TestLibCommonNetwork)
{
public:

	TEST_METHOD(ConvertSlashEightPrefix)
	{
		Assert::AreEqual(::common::network::MaskFromRoutingPrefix(8), 0xFF000000);
	}

	TEST_METHOD(ConvertSlashTwelvePrefix)
	{
		Assert::AreEqual(::common::network::MaskFromRoutingPrefix(12), 0xFFF00000);
	}

	TEST_METHOD(ConvertSlashThirtyOnePrefix)
	{
		Assert::AreEqual(::common::network::MaskFromRoutingPrefix(31), 0xFFFFFFFE);
	}

	TEST_METHOD(ConvertSlashThirtyTwoPrefix)
	{
		Assert::AreEqual(::common::network::MaskFromRoutingPrefix(32), 0xFFFFFFFF);
	}

	TEST_METHOD(ConvertSlashZeroPrefix)
	{
		Assert::AreEqual(::common::network::MaskFromRoutingPrefix(0), 0U);
	}

};

}
