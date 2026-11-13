# SQLCipher must use CommonCrypto on Apple platforms, but libsqlite3-sys links OpenSSL instead
# when any of these variables is set, with or without an uppercased target triple prefix.

OPENSSL_ENV_VARS := OPENSSL_DIR OPENSSL_LIB_DIR OPENSSL_INCLUDE_DIR
OPENSSL_ENV_TARGET_PREFIXES := AARCH64_APPLE_DARWIN X86_64_APPLE_DARWIN AARCH64_APPLE_IOS \
	AARCH64_APPLE_IOS_SIM X86_64_APPLE_IOS

unexport $(OPENSSL_ENV_VARS) \
	$(foreach p,$(OPENSSL_ENV_TARGET_PREFIXES),$(addprefix $(p)_,$(OPENSSL_ENV_VARS)))
