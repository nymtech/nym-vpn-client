#include "stdafx.h"
#include "CppUnitTest.h"
#include "libwfp/conditionbuilder.h"
#include "libwfp/conditions/conditionapplication.h"
#include "libwfp/conditions/conditionport.h"
#include "libwfp/filterbuilder.h"
#include "libwfp/sublayerbuilder.h"
#include "libwfp/providerbuilder.h"
#include "libwfp/filterengine.h"
#include "libwfp/objectexplorer.h"
#include "libwfp/objectdeleter.h"
#include "libwfp/objectinstaller.h"
#include "libwfp/transaction.h"
#include "libcommon/string.h"
#include "test-libwfp/ncat.h"
#include <memory>

using namespace Microsoft::VisualStudio::CppUnitTestFramework;
using namespace ::wfp::conditions;

namespace testlibwfp
{

TEST_CLASS(IntegrationTest)
{
public:

	TEST_METHOD(BasicFilterOps)
	{
		static const GUID Layer = FWPM_LAYER_ALE_FLOW_ESTABLISHED_V4;

		wfp::ConditionBuilder conditionBuilder(Layer);
		conditionBuilder.add_condition(ConditionPort::Remote(5555));

		wfp::FilterBuilder filterBuilder;

		static const GUID FilterKey = {
			0xa86fd1bf,
			0x21cd,
			0x6432,
			{ 0xa0, 0xbb, 0x17, 0x42, 0x5c, 0x88, 0x5c, 0x58 }
		};

		filterBuilder
			.key(FilterKey)
			.name(L"BasicFilterOps")
			.description(L"Integration test: Block outgoing traffic with destination port 5555")
			.layer(Layer)
			.weight(MAXUINT64)
			.block();

		auto engine = wfp::FilterEngine::DynamicSession();

		Assert::IsTrue(wfp::ObjectInstaller::AddFilter(*engine, filterBuilder, conditionBuilder));

		Assert::IsTrue(wfp::ObjectExplorer::GetFilter(*engine, FilterKey, [](const FWPM_FILTER0 &filter)
		{
			return 0 == wcscmp(filter.displayData.name, L"BasicFilterOps");
		}));

		wfp::ObjectDeleter::DeleteFilter(*engine, FilterKey);

		Assert::IsFalse(wfp::ObjectExplorer::GetFilter(*engine, FilterKey, [](const FWPM_FILTER0 &)
		{
			return true;
		}));
	}

	TEST_METHOD(BasicProviderOps)
	{
		static const GUID ProviderKey = {
			0xa86fd1bf,
			0x21d0,
			0x6432,
			{ 0xa0, 0xbb, 0x17, 0x42, 0x5c, 0x88, 0x5c, 0x58 }
		};

		static const uint8_t ProviderData[]{
			0xaa, 0xbb, 0xcc, 0xdd, 0xee
		};

		wfp::ProviderBuilder providerBuilder;

		providerBuilder
			.key(ProviderKey)
			.name(L"BasicProviderOps")
			.description(L"Integration test: Dummy provider")
			.data(ProviderData, sizeof(ProviderData));

		auto engine = wfp::FilterEngine::DynamicSession();

		Assert::IsTrue(wfp::ObjectInstaller::AddProvider(*engine, providerBuilder));

		Assert::IsTrue(wfp::ObjectExplorer::GetProvider(*engine, ProviderKey, [](const FWPM_PROVIDER0 &provider)
			{
				return 0 == wcscmp(provider.displayData.name, L"BasicProviderOps")
					&& sizeof(ProviderData) == provider.providerData.size
					&& 0 == memcmp(provider.providerData.data, ProviderData, sizeof(ProviderData));
			}));

		wfp::ObjectDeleter::DeleteProvider(*engine, ProviderKey);

		Assert::IsFalse(wfp::ObjectExplorer::GetProvider(*engine, ProviderKey, [](const FWPM_PROVIDER0 &)
		{
			return true;
		}));
	}

	TEST_METHOD(BasicSublayerOps)
	{
		static const GUID SublayerKey = {
			0xa86fd1bf,
			0x21d0,
			0x6433,
			{ 0xa0, 0xbb, 0x17, 0x42, 0x5c, 0x88, 0x5c, 0x58 }
		};

		// Context data is always referred to as "provider data".
		static const uint8_t ProviderData[]{
			0xaa, 0xbb, 0xcc, 0xdd, 0xee
		};

		wfp::SublayerBuilder sublayerBuilder;

		sublayerBuilder
			.key(SublayerKey)
			.name(L"BasicSublayerOps")
			.description(L"Integration test: Dummy sublayer")
			.data(ProviderData, sizeof(ProviderData));

		auto engine = wfp::FilterEngine::DynamicSession();

		Assert::IsTrue(wfp::ObjectInstaller::AddSublayer(*engine, sublayerBuilder));

		Assert::IsTrue(wfp::ObjectExplorer::GetSublayer(*engine, SublayerKey, [](const FWPM_SUBLAYER0 &sublayer)
		{
			return 0 == wcscmp(sublayer.displayData.name, L"BasicSublayerOps")
				&& sizeof(ProviderData) == sublayer.providerData.size
				&& 0 == memcmp(sublayer.providerData.data, ProviderData, sizeof(ProviderData));
		}));

		wfp::ObjectDeleter::DeleteSublayer(*engine, SublayerKey);

		Assert::IsFalse(wfp::ObjectExplorer::GetSublayer(*engine, SublayerKey, [](const FWPM_SUBLAYER0 &)
		{
			return true;
		}));
	}

	TEST_METHOD(BlockOutgoingByPort)
	{
		static const GUID ProviderKey = {
			0xa86fd1bf,
			0x21d0,
			0x0000,
			{ 0xa0, 0xbb, 0x17, 0x42, 0x5c, 0x88, 0x5c, 0x58 }
		};

		static const uint8_t ProviderData[]{
			0xaa, 0xbb, 0xcc, 0xdd, 0xee
		};

		wfp::ProviderBuilder providerBuilder;

		providerBuilder
			.key(ProviderKey)
			.name(L"BlockOutgoingByPort")
			.description(L"Integration test: Block Ncat outgoing traffic with destination port 80")
			.data(ProviderData, sizeof(ProviderData));

		auto engine = wfp::FilterEngine::DynamicSession();

		Assert::IsTrue(wfp::ObjectInstaller::AddProvider(*engine, providerBuilder));

		static const GUID SublayerKey = {
			0xa86fd1bf,
			0x21d0,
			0x0001,
			{ 0xa0, 0xbb, 0x17, 0x42, 0x5c, 0x88, 0x5c, 0x58 }
		};

		// Context data is always referred to as "provider data".
		static const uint8_t SublayerProviderData[]{
			0xaa, 0xbb, 0xcc, 0xdd, 0xef
		};

		wfp::SublayerBuilder sublayerBuilder;

		sublayerBuilder
			.key(SublayerKey)
			.name(L"BlockOutgoingByPort")
			.description(L"Integration test: Block Ncat outgoing traffic with destination port 80")
			.provider(ProviderKey)
			.data(SublayerProviderData, sizeof(SublayerProviderData))
			.weight(MAXUINT16);

		Assert::IsTrue(wfp::ObjectInstaller::AddSublayer(*engine, sublayerBuilder));

		static const unsigned short PORT = 80;
		static const GUID LayerKey = FWPM_LAYER_ALE_FLOW_ESTABLISHED_V4;

		static const GUID FilterKey = {
			0xa86fd1bf,
			0x21d0,
			0x0002,
			{ 0xa0, 0xbb, 0x17, 0x42, 0x5c, 0x88, 0x5c, 0x58 }
		};

		wfp::ConditionBuilder conditionBuilder(LayerKey);

		conditionBuilder.add_condition(std::make_unique<ConditionApplication>(Ncat::Path()));
		conditionBuilder.add_condition(ConditionPort::Remote(PORT));

		wfp::FilterBuilder filterBuilder;

		filterBuilder
			.key(FilterKey)
			.name(L"BlockOutgoingByPort")
			.description(L"Integration test: Block Ncat outgoing traffic with destination port 80")
			.provider(ProviderKey)
			.layer(LayerKey)
			.sublayer(SublayerKey)
			.weight(MAXUINT64)
			.block();

		Assert::IsTrue(wfp::ObjectInstaller::AddFilter(*engine, filterBuilder, conditionBuilder));

		Ncat ncat(L"-4 www.mullvad.net 80");

		// Verify error message was printed
		Assert::IsTrue(common::string::BeginsWith(ncat.output(), std::string("Ncat:")));

		// Verify return code
		Assert::AreEqual(ncat.returnCode(), (DWORD)Ncat::ReturnCode::Error);
	}

};

}
