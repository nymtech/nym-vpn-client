// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::borrow::Cow;

#[derive(Default)]
pub struct UapiConfigBuilder {
    buf: Vec<u8>,
}

impl UapiConfigBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add<'a, C: Into<Value<'a>> + 'a>(&mut self, key: &str, value: C) -> &mut Self {
        self.buf.extend(key.as_bytes());
        self.buf.extend(b"=");
        self.buf.extend(value.into().to_bytes().as_ref());
        self.buf.extend(b"\n");
        self
    }

    pub fn into_bytes(mut self) -> Vec<u8> {
        self.buf.push(b'\n');
        self.buf
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

impl<'a> Value<'a> {
    fn to_bytes(&self) -> Cow<'a, [u8]> {
        match self {
            Value::String(s) => s.as_bytes().into(),
            Value::Bytes(bytes) => Cow::Owned(hex::encode(bytes).into_bytes()),
        }
    }
}

#[cfg(test)]
pub mod tests {
    use super::UapiConfigBuilder;

    #[test]
    fn test_encode_string() {
        let mut config_builder = UapiConfigBuilder::new();
        config_builder.add("key", "value");
        assert_eq!(config_builder.into_bytes(), b"key=value\n\n");
    }

    #[test]
    fn test_encode_bytes_with_hex() {
        let mut config_builder: UapiConfigBuilder = UapiConfigBuilder::new();
        config_builder.add("key", "bytes".as_bytes());
        assert_eq!(config_builder.into_bytes(), b"key=6279746573\n\n");
    }
}
