#include "stdafx.h"
#include "CppUnitTest.h"
#include "libwfp/filterengine.h"
#include "libwfp/objectinstaller.h"
#include "libwfp/objectdeleter.h"
#include "libwfp/objectexplorer.h"
#include "libwfp/providerbuilder.h"
#include "libwfp/transaction.h"

using namespace Microsoft::VisualStudio::CppUnitTestFramework;

namespace testlibwfp
{

TEST_CLASS(TransactionIntegrationTest)
{
public:

	TEST_METHOD(ReadWriteTransaction)
	{
		static const GUID ProviderKey = {
			0xa86fd1bf,
			0x21d0,
			0x6432,
			{ 0xa0, 0xbb, 0x17, 0x42, 0x5c, 0x88, 0x5c, 0x58 }
		};

		wfp::ProviderBuilder builder;

		builder
			.key(ProviderKey)
			.name(L"ReadWriteTransaction")
			.description(L"Integration test: Dummy provider");

		auto engine = wfp::FilterEngine::DynamicSession();

		auto providerOps = [&]()
		{
			return wfp::ObjectInstaller::AddProvider(*engine, builder)
				&& wfp::ObjectExplorer::GetProvider(*engine, ProviderKey, [](const FWPM_PROVIDER0 &)
			{
				return true;
			});
		};

		Assert::IsTrue(wfp::Transaction::Execute(*engine, providerOps));
	}

	TEST_METHOD(ReadOnlyTransaction)
	{
		static const GUID ProviderKey = {
			0xa86fd1bf,
			0x21d0,
			0x6432,
			{ 0xa0, 0xbb, 0x17, 0x42, 0x5c, 0x88, 0x5c, 0x58 }
		};

		wfp::ProviderBuilder builder;

		builder
			.key(ProviderKey)
			.name(L"ReadOnlyTransaction")
			.description(L"Integration test: Dummy provider");

		auto engine = wfp::FilterEngine::DynamicSession();

		Assert::IsTrue(wfp::ObjectInstaller::AddProvider(*engine, builder));

		auto verifyProvider = [&]()
		{
			return wfp::ObjectExplorer::GetProvider(*engine, ProviderKey, [](const FWPM_PROVIDER0 &provider)
			{
				return 0 == wcscmp(provider.displayData.name, L"ReadOnlyTransaction");
			});
		};

		Assert::IsTrue(wfp::Transaction::ExecuteReadOnly(*engine, verifyProvider));
	}

	TEST_METHOD(ReadOnlyTransactionIsReadOnly)
	{
		auto engine = wfp::FilterEngine::DynamicSession();

		auto addProvider = [&]()
		{
			wfp::ProviderBuilder builder;

			builder
				.name(L"ReadOnlyTransactionIsReadOnly")
				.description(L"Integration test: Dummy provider");

			return wfp::ObjectInstaller::AddProvider(*engine, builder);
		};

		Assert::ExpectException<std::exception>([&]() { wfp::Transaction::ExecuteReadOnly(*engine, addProvider); });
	}

};

}
