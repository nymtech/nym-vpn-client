#include "pch.h"
#include "libcommon/math.h"
#include "CppUnitTest.h"

using namespace Microsoft::VisualStudio::CppUnitTestFramework;

namespace testlibcommon
{

TEST_CLASS(TestLibCommonMath)
{
public:

	TEST_METHOD(RoundPowerTwoAlreadyEvenMultiple)
	{
		Assert::AreEqual(
			::common::math::RoundPowerTwo(sizeof(size_t), sizeof(size_t)),
			sizeof(size_t)
		);
	}

	TEST_METHOD(RoundPowerTwoToNextMultiple)
	{
		Assert::AreEqual(
			::common::math::RoundPowerTwo(sizeof(size_t) + 1, sizeof(size_t)),
			sizeof(size_t) * 2
		);
	}

	TEST_METHOD(RoundPowerTwoToNextMultipleAgain)
	{
		Assert::AreEqual(
			::common::math::RoundPowerTwo((sizeof(size_t) * 2) - 1, sizeof(size_t)),
			sizeof(size_t) * 2
		);
	}
};

}
