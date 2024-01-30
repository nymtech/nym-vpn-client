#include "stdafx.h"
#include "CppUnitTest.h"
#include "libwfp/ipaddress.h"
#include "libwfp/ipnetwork.h"

using IpAddress = ::wfp::IpAddress;
using IpNetwork = ::wfp::IpNetwork;

using namespace Microsoft::VisualStudio::CppUnitTestFramework;

namespace Microsoft::VisualStudio::CppUnitTestFramework
{

template<>
std::wstring ToString<wfp::IpAddress::Type>(const wfp::IpAddress::Type &address)
{
	switch (address)
	{
		case IpAddress::Type::Ipv4:
			return L"IpAddress::Type::Ipv4";
		case IpAddress::Type::Ipv6:
			return L"IpAddress::Type::Ipv6";
	}
	return L"IpAddress::Type::UNKNOWN";
}

}

namespace testlibwfp
{

TEST_CLASS(TestLibWfpIpAddr)
{
public:

	TEST_METHOD(ConstructIpv4)
	{
		static const wchar_t *literalString = L"127.0.0.1";

		IpAddress ip_from_literal(IpAddress::Literal({ 127, 0, 0, 1 }));
		IpAddress ip_from_string(literalString);
		IpAddress ip_from_raw(0x7f000001);

		Assert::AreEqual(ip_from_literal.type(), IpAddress::Type::Ipv4);
		Assert::AreEqual(ip_from_string.type(), IpAddress::Type::Ipv4);
		Assert::AreEqual(ip_from_raw.type(), IpAddress::Type::Ipv4);

		Assert::AreEqual(ip_from_string.toString().c_str(), literalString);
		Assert::AreEqual(ip_from_literal.toString().c_str(), literalString);
		Assert::AreEqual(ip_from_raw.toString().c_str(), literalString);
	}

	TEST_METHOD(ConstructIpv4NetworkIncludes)
	{
		static const uint32_t prefix = 16;
		IpAddress network_address(IpAddress::Literal{ 192, 168, 0, 0 });
		IpNetwork network(network_address, prefix);

		const IpAddress::Literal matchingAddresses[] = {
			IpAddress::Literal { 192, 168, 0, 1 },
			IpAddress::Literal { 192, 168, 254, 254 }
		};

		for (const auto &address : matchingAddresses)
		{
			Assert::IsTrue(network.includes(address));
		}

		const IpAddress::Literal nonMatchingAddresses[] = {
			IpAddress::Literal { 192, 169, 0, 1 },
			IpAddress::Literal { 193, 168, 254, 254 },
			IpAddress::Literal { 127, 0, 0, 1 }
		};

		for (const auto &address : nonMatchingAddresses)
		{
			Assert::IsFalse(network.includes(address));
		}
	}

	TEST_METHOD(ConstructIpv6NetworkIncludes)
	{
		static const uint32_t prefix = 10;
		IpAddress network_address(IpAddress::Literal6{ 0xfe80, 0, 0, 0, 0, 0, 0, 0 });
		IpNetwork network(network_address, prefix);

		const IpAddress::Literal6 matchingAddresses[] = {
			IpAddress::Literal6 { 0xfe80, 0, 0, 0, 0, 0, 0, 0 },
			// Test trailing bits
			IpAddress::Literal6 { 0xfebf, 0, 0, 0, 0, 0, 0, 0 },
			IpAddress::Literal6 { 0xfe80, 0, 1, 0, 1, 0, 1, 0 }
		};

		for (const auto &address : matchingAddresses)
		{
			Assert::IsTrue(network.includes(address));
		}

		const IpAddress::Literal6 nonMatchingAddresses[] = {
			IpAddress::Literal6 { 0, 1, 0, 1, 0, 1, 0, 1 },
			// Test trailing bits
			IpAddress::Literal6 { 0xfec0, 0, 0, 0, 0, 0, 0, 0 }
		};

		for (const auto &address : nonMatchingAddresses)
		{
			Assert::IsFalse(network.includes(address));
		}
	}

	TEST_METHOD(ConstructIpv6)
	{
		static const wchar_t *literalString = L"1000:1100:1200:1300:1400:1500:1600:1700";

		IpAddress ip_from_literal(IpAddress::Literal6({ 0x1000, 0x1100, 0x1200, 0x1300, 0x1400, 0x1500, 0x1600, 0x1700 }));
		IpAddress ip_from_string(literalString);

		const uint8_t raw[16] = { 0x10, 00, 0x11, 00, 0x12, 00, 0x13, 00, 0x14, 00, 0x15, 00, 0x16, 00, 0x17, 00 };

		FWP_BYTE_ARRAY16 netnode;
		memcpy(netnode.byteArray16, raw, sizeof(netnode.byteArray16));

		IpAddress ip_from_raw(netnode);

		Assert::AreEqual(ip_from_literal.type(), IpAddress::Type::Ipv6);
		Assert::AreEqual(ip_from_string.type(), IpAddress::Type::Ipv6);
		Assert::AreEqual(ip_from_raw.type(), IpAddress::Type::Ipv6);

		Assert::AreEqual(ip_from_string.toString().c_str(), literalString);
		Assert::AreEqual(ip_from_literal.toString().c_str(), literalString);
		Assert::AreEqual(ip_from_raw.toString().c_str(), literalString);
	}

};

}
