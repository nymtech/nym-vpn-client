package net.nymtech.nymvpn.util

import org.junit.Assert.assertEquals
import org.junit.Test

class FreepassCodeTest {
	private fun valid(s: String) = parseFreepassCode(s) as? FreepassParseResult.Valid

	@Test fun bareCode() = assertEquals("eJMWikx3EeU", valid("eJMWikx3EeU")?.code)
	@Test fun bareCodeTrimmed() = assertEquals("eJMWikx3EeU", valid("  eJMWikx3EeU \n")?.code)
	@Test fun trustedUrl() = assertEquals("eJMWikx3EeU", valid("https://nym.com/account/freepass?code=eJMWikx3EeU")?.code)
	@Test fun trustedSubdomain() = assertEquals("eJMWikx3EeU", valid("https://sub.nym.com/x?code=eJMWikx3EeU")?.code)
	@Test fun trustedUrlHostCaseInsensitive() = assertEquals("eJMWikx3EeU", valid("https://NYM.com/?code=eJMWikx3EeU")?.code)

	@Test fun rejectsUntrustedHost() = assertEquals(FreepassParseResult.Invalid, parseFreepassCode("https://evil.com/?code=eJMWikx3EeU"))
	@Test fun rejectsLookalikeHost() = assertEquals(FreepassParseResult.Invalid, parseFreepassCode("https://nym.com.evil.com/?code=eJMWikx3EeU"))
	@Test fun rejectsHttpScheme() = assertEquals(FreepassParseResult.Invalid, parseFreepassCode("http://nym.com/?code=eJMWikx3EeU"))
	@Test fun rejectsJavascriptScheme() = assertEquals(FreepassParseResult.Invalid, parseFreepassCode("javascript:alert(1)"))
	@Test fun rejectsFileScheme() = assertEquals(FreepassParseResult.Invalid, parseFreepassCode("file:///etc/passwd"))
	@Test fun rejectsMissingCodeParam() = assertEquals(FreepassParseResult.Invalid, parseFreepassCode("https://nym.com/account/freepass"))
	@Test fun rejectsNonBase58() = assertEquals(FreepassParseResult.Invalid, parseFreepassCode("abc0OIl"))
	@Test fun rejectsSymbols() = assertEquals(FreepassParseResult.Invalid, parseFreepassCode("abc';DROP"))
	@Test fun rejectsTooShort() = assertEquals(FreepassParseResult.Invalid, parseFreepassCode("ab"))
	@Test fun rejectsTooLong() = assertEquals(FreepassParseResult.Invalid, parseFreepassCode("a".repeat(129)))
	@Test fun rejectsInternalWhitespace() = assertEquals(FreepassParseResult.Invalid, parseFreepassCode("eJMW ikx3EeU"))
	@Test fun rejectsControlChars() = assertEquals(FreepassParseResult.Invalid, parseFreepassCode("eJMW ikx3"))
	@Test fun rejectsOversizedBlob() = assertEquals(FreepassParseResult.Invalid, parseFreepassCode("x".repeat(5000)))
	@Test fun rejectsEmpty() = assertEquals(FreepassParseResult.Invalid, parseFreepassCode(""))
}
