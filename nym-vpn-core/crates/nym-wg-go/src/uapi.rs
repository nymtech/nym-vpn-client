// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use zeroize::Zeroizing;

#[derive(Default)]
pub struct UapiConfigBuilder {
    entries: Vec<UapiEntry>,
}

impl UapiConfigBuilder {
    pub fn add<C: Into<Value>>(&mut self, key: &'static str, value: C) -> &mut Self {
        self.entries.push(UapiEntry {
            key,
            value: value.into(),
        });
        self
    }

    /// Returns nul-terminated buffer holding UAPI configuration
    pub fn into_bytes_with_nul(self) -> Zeroizing<Vec<u8>> {
        let capacity = self
            .entries
            .iter()
            // +1 for the newline terminating each entry
            .map(|entry| entry.byte_length() + 1)
            .sum::<usize>()
            // +2 for the trailing empty line and nul terminator
            + 2;

        let mut buf = Zeroizing::new(Vec::with_capacity(capacity));
        for entry in &self.entries {
            entry.append_to(&mut buf);
            buf.extend(b"\n");
        }
        buf.extend(b"\n\0");
        debug_assert_eq!(buf.len(), capacity);
        buf
    }
}

struct UapiEntry {
    key: &'static str,
    value: Value,
}

impl UapiEntry {
    /// Returns the length of the encoded `key=value` pair, excluding the trailing newline
    fn byte_length(&self) -> usize {
        // 1 is the length of `=` key value separator
        self.key.len() + self.value.as_bytes().len() + 1
    }

    fn append_to(&self, buf: &mut Vec<u8>) {
        buf.extend(self.key.as_bytes());
        buf.extend(b"=");
        buf.extend(self.value.as_bytes());
    }
}

pub struct Value(Zeroizing<Vec<u8>>);

impl From<&str> for Value {
    fn from(s: &str) -> Value {
        Value(Zeroizing::new(s.as_bytes().to_vec()))
    }
}

impl From<&[u8]> for Value {
    fn from(s: &[u8]) -> Value {
        let mut encoded = Zeroizing::new(vec![0; s.len() * 2]);
        hex::encode_to_slice(s, &mut encoded).expect("output slice is twice the input length");
        Value(encoded)
    }
}

impl Value {
    fn as_bytes(&self) -> &[u8] {
        &self.0
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
