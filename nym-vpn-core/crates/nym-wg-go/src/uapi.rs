// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use zeroize::{ZeroizeOnDrop, Zeroizing};

#[derive(ZeroizeOnDrop)]
pub struct UapiConfigBuilder {
    buf: Vec<u8>,
}

impl Default for UapiConfigBuilder {
    fn default() -> Self {
        Self {
            // Amnezia, two catch-all allowed_ips and ipv6 endpoint come at around 500 bytes
            // Let's use 1024 to provide enough headroom for the average config to avoid reallocations
            buf: Vec::with_capacity(1024),
        }
    }
}

impl UapiConfigBuilder {
    pub fn add<'a, C: Into<Value<'a>> + 'a>(&mut self, key: &str, value: C) -> &mut Self {
        self.buf.extend(key.as_bytes());
        self.buf.extend(b"=");
        value.into().append_to(&mut self.buf);
        self.buf.extend(b"\n");
        self
    }

    /// Returns nul-terminated buffer holding UAPI configuration
    pub fn into_bytes_with_nul(mut self) -> Zeroizing<Vec<u8>> {
        self.buf.extend(b"\n\0");
        Zeroizing::new(std::mem::take(&mut self.buf))
    }
}

pub enum Value<'a> {
    String(&'a str),
    Bytes(&'a [u8]),
}

impl<'a> From<&'a str> for Value<'a> {
    fn from(s: &'a str) -> Value<'a> {
        Value::String(s)
    }
}

impl<'a> From<&'a [u8]> for Value<'a> {
    fn from(s: &'a [u8]) -> Value<'a> {
        Value::Bytes(s)
    }
}

impl Value<'_> {
    fn append_to(self, buf: &mut Vec<u8>) {
        match self {
            Value::String(s) => buf.extend(s.as_bytes()),
            Value::Bytes(bytes) => {
                let start = buf.len();
                // encode_to_slice() expects the output to hold at least `bytes.len() * 2` bytes
                buf.resize(start + bytes.len() * 2, 0);
                // SAFETY: buf.resize() ensures that there is enough space in the `buf`
                hex::encode_to_slice(bytes, &mut buf[start..])
                    .expect("output slice is twice the input length");
            }
        }
    }
}

#[cfg(test)]
pub mod tests {
    use super::UapiConfigBuilder;

    #[test]
    fn test_encode_string() {
        let mut config_builder = UapiConfigBuilder::default();
        config_builder.add("key", "value");
        assert_eq!(
            config_builder.into_bytes_with_nul().as_slice(),
            b"key=value\n\n\0"
        );
    }

    #[test]
    fn test_encode_bytes_with_hex() {
        let mut config_builder = UapiConfigBuilder::default();
        config_builder.add("key", "bytes".as_bytes());
        assert_eq!(
            config_builder.into_bytes_with_nul().as_slice(),
            b"key=6279746573\n\n\0"
        );
    }
}
