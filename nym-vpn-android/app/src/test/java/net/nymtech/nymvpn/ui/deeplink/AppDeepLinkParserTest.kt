package net.nymtech.nymvpn.ui.deeplink

import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Test

class AppDeepLinkParserTest {

	private val id = "1234567890"
	private val payload = "abcDEF123".repeat(16)
	private val params = "deeplink_id=$id&payload=$payload"

	private val privyHttps = "https://nym.com/app/privy/return"
	private val paymentHttps = "https://nym.com/app/payment/return"
	private val privyLegacy = "nymvpn://auth/privy/privateKey"
	private val paymentLegacy = "nymvpn://account/response"

	private val privyCallback = "$privyLegacy?$params"
	private val paymentCallback = "$paymentLegacy?$params"

	@Test
	fun `https privy return with params in fragment`() {
		assertEquals(AppDeepLink.PrivyReturn(privyCallback), AppDeepLinkParser.parse("$privyHttps#$params"))
	}

	@Test
	fun `https privy return with params in query`() {
		assertEquals(AppDeepLink.PrivyReturn(privyCallback), AppDeepLinkParser.parse("$privyHttps?$params"))
	}

	@Test
	fun `https privy return with reordered params is rebuilt in canonical order`() {
		assertEquals(
			AppDeepLink.PrivyReturn(privyCallback),
			AppDeepLinkParser.parse("$privyHttps#payload=$payload&deeplink_id=$id"),
		)
	}

	@Test
	fun `https privy return with url-encoded deeplink_id`() {
		assertEquals(
			AppDeepLink.PrivyReturn(privyCallback),
			AppDeepLinkParser.parse("$privyHttps#deeplink_%69d=%31234567890&payload=$payload"),
		)
	}

	@Test
	fun `legacy privy return`() {
		assertEquals(AppDeepLink.PrivyReturn(privyCallback), AppDeepLinkParser.parse("$privyLegacy?$params"))
	}

	@Test
	fun `legacy privy return with trailing slash`() {
		assertEquals(AppDeepLink.PrivyReturn(privyCallback), AppDeepLinkParser.parse("$privyLegacy/?$params"))
	}

	@Test
	fun `https payment return without params`() {
		assertEquals(AppDeepLink.PaymentReturn(null), AppDeepLinkParser.parse(paymentHttps))
	}

	@Test
	fun `https payment return with account payload`() {
		assertEquals(AppDeepLink.PaymentReturn(paymentCallback), AppDeepLinkParser.parse("$paymentHttps#$params"))
	}

	@Test
	fun `legacy payment return without params`() {
		assertEquals(AppDeepLink.PaymentReturn(null), AppDeepLinkParser.parse(paymentLegacy))
	}

	@Test
	fun `legacy payment return with account payload`() {
		assertEquals(AppDeepLink.PaymentReturn(paymentCallback), AppDeepLinkParser.parse("$paymentLegacy?$params"))
	}

	@Test
	fun `rejects null and empty`() {
		assertNull(AppDeepLinkParser.parse(null))
		assertNull(AppDeepLinkParser.parse(""))
	}

	@Test
	fun `rejects http`() {
		assertNull(AppDeepLinkParser.parse("http://nym.com/app/privy/return#$params"))
	}

	@Test
	fun `rejects other hosts`() {
		assertNull(AppDeepLinkParser.parse("https://evil.com/app/privy/return#$params"))
		assertNull(AppDeepLinkParser.parse("https://nym.com.evil.com/app/privy/return#$params"))
		assertNull(AppDeepLinkParser.parse("https://evil.nym.com/app/privy/return#$params"))
		assertNull(AppDeepLinkParser.parse("https://www.nym.com/app/privy/return#$params"))
	}

	@Test
	fun `rejects user info and port`() {
		assertNull(AppDeepLinkParser.parse("https://user@nym.com/app/privy/return#$params"))
		assertNull(AppDeepLinkParser.parse("https://nym.com:8443/app/privy/return#$params"))
	}

	@Test
	fun `rejects other schemes`() {
		assertNull(AppDeepLinkParser.parse("intent://auth/privy/privateKey?$params"))
		assertNull(AppDeepLinkParser.parse("javascript:alert(1)"))
		assertNull(AppDeepLinkParser.parse("file:///data/local/tmp/x"))
	}

	@Test
	fun `rejects wrong https paths`() {
		assertNull(AppDeepLinkParser.parse("https://nym.com/app/privy/return/#$params"))
		assertNull(AppDeepLinkParser.parse("https://nym.com/app/privy/return/x#$params"))
		assertNull(AppDeepLinkParser.parse("https://nym.com/APP/privy/return#$params"))
		assertNull(AppDeepLinkParser.parse("https://nym.com/app/privy/../privy/return#$params"))
		assertNull(AppDeepLinkParser.parse("https://nym.com/app/privy/%72eturn#$params"))
	}

	@Test
	fun `rejects legacy host and path mix-ups`() {
		assertNull(AppDeepLinkParser.parse("nymvpn://account/privy/privateKey?$params"))
		assertNull(AppDeepLinkParser.parse("nymvpn://auth/response?$params"))
		assertNull(AppDeepLinkParser.parse("nymvpn://auth/privy/privateKeyX?$params"))
	}

	@Test
	fun `rejects privy return without params`() {
		assertNull(AppDeepLinkParser.parse(privyHttps))
		assertNull(AppDeepLinkParser.parse(privyLegacy))
	}

	@Test
	fun `rejects missing param`() {
		assertNull(AppDeepLinkParser.parse("$privyHttps#deeplink_id=$id"))
		assertNull(AppDeepLinkParser.parse("$privyHttps#payload=$payload"))
	}

	@Test
	fun `rejects duplicate param`() {
		assertNull(AppDeepLinkParser.parse("$privyHttps#$params&deeplink_id=1"))
		assertNull(AppDeepLinkParser.parse("$privyHttps#deeplink_id=$id&deeplink_id=$id"))
	}

	@Test
	fun `rejects extra param`() {
		assertNull(AppDeepLinkParser.parse("$privyHttps#$params&redirect=https://evil.com"))
		assertNull(AppDeepLinkParser.parse("$privyLegacy?$params&x=1"))
	}

	@Test
	fun `rejects params in both fragment and query`() {
		assertNull(AppDeepLinkParser.parse("$privyHttps?$params#$params"))
		assertNull(AppDeepLinkParser.parse("$privyLegacy?$params#x"))
	}

	@Test
	fun `rejects malformed pairs`() {
		assertNull(AppDeepLinkParser.parse("$privyHttps#deeplink_id&payload=$payload"))
		assertNull(AppDeepLinkParser.parse("$privyHttps#&$params"))
	}

	@Test
	fun `rejects invalid deeplink_id`() {
		assertNull(AppDeepLinkParser.parse("$privyHttps#deeplink_id=abc&payload=$payload"))
		assertNull(AppDeepLinkParser.parse("$privyHttps#deeplink_id=-1&payload=$payload"))
		assertNull(AppDeepLinkParser.parse("$privyHttps#deeplink_id=${"1".repeat(21)}&payload=$payload"))
		assertNull(AppDeepLinkParser.parse("$privyHttps#deeplink_id=&payload=$payload"))
		assertNull(AppDeepLinkParser.parse("$privyHttps#deeplink_id=1%26x%3D1&payload=$payload"))
	}

	@Test
	fun `rejects invalid payload`() {
		// base58 excludes 0, O, I, l
		assertNull(AppDeepLinkParser.parse("$privyHttps#deeplink_id=$id&payload=0${payload.drop(1)}"))
		assertNull(AppDeepLinkParser.parse("$privyHttps#deeplink_id=$id&payload=${"a".repeat(99)}"))
		assertNull(AppDeepLinkParser.parse("$privyHttps#deeplink_id=$id&payload=${"a".repeat(201)}"))
		assertNull(AppDeepLinkParser.parse("$privyHttps#deeplink_id=$id&payload=$payload%00"))
		assertNull(AppDeepLinkParser.parse("$privyHttps#deeplink_id=$id&payload=$payload%0A"))
	}

	@Test
	fun `rejects invalid payment payload instead of ignoring it`() {
		assertNull(AppDeepLinkParser.parse("$paymentHttps#deeplink_id=abc&payload=$payload"))
		assertNull(AppDeepLinkParser.parse("$paymentLegacy?foo=bar"))
	}

	@Test
	fun `rejects oversized input`() {
		assertNull(AppDeepLinkParser.parse("$privyHttps#$params&${"x".repeat(2048)}"))
	}
}
