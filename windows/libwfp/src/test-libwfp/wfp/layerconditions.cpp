#include "stdafx.h"
#include "CppUnitTest.h"
#include "libwfp/layerconditions.h"
#include <guiddef.h>
#include <fwpmu.h>

using namespace Microsoft::VisualStudio::CppUnitTestFramework;

namespace testlibwfp
{

TEST_CLASS(TestLibWfpLayerConditions)
{
public:

	TEST_METHOD(IsCompatibleWithCompatibleCondition)
	{
		Assert::IsTrue(wfp::LayerConditions::IsCompatible(FWPM_LAYER_OUTBOUND_TRANSPORT_V6, FWPM_CONDITION_IP_LOCAL_ADDRESS_TYPE));

		// Match last item in defined condition array, to ensure no off-by-one issues
		Assert::IsTrue(wfp::LayerConditions::IsCompatible(FWPM_LAYER_OUTBOUND_TRANSPORT_V6, FWPM_CONDITION_CURRENT_PROFILE_ID));
	}

	TEST_METHOD(IsNotCompatibleWithIncompatibleCondition)
	{
		Assert::IsFalse(wfp::LayerConditions::IsCompatible(FWPM_LAYER_OUTBOUND_TRANSPORT_V6, FWPM_CONDITION_DIRECTION));
	}

	TEST_METHOD(IsCompatibleWithInvalidLayerThrows)
	{
		static const GUID InvalidLayer = {
			0xa86fd1bf,
			0x21cd,
			0x497e,
			{ 0xa0, 0xbb, 0x17, 0x42, 0x5c, 0x88, 0x5c, 0x58 }
		};

		Assert::ExpectException<std::runtime_error>([&]() { wfp::LayerConditions::IsCompatible(InvalidLayer, FWPM_CONDITION_DIRECTION); });
	}

};

}
