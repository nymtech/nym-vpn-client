// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use minicbor::{Decode, Decoder, Encode, Encoder, decode, encode};

use crate::{NodeIdentity, Recipient};

impl<C> Encode<C> for NodeIdentity {
    fn encode<W: encode::Write>(
        &self,
        e: &mut Encoder<W>,
        _: &mut C,
    ) -> Result<(), encode::Error<W::Error>> {
        e.str(&self.to_base58_string())?.ok()
    }
}

impl<'b, C> Decode<'b, C> for NodeIdentity {
    fn decode(d: &mut Decoder<'b>, _: &mut C) -> Result<Self, decode::Error> {
        let position = d.position();
        Self::from_base58_string(d.str()?).map_err(|e| decode::Error::custom(e).at(position))
    }
}

impl<C> Encode<C> for Recipient {
    fn encode<W: encode::Write>(
        &self,
        e: &mut Encoder<W>,
        _: &mut C,
    ) -> Result<(), encode::Error<W::Error>> {
        e.str(&self.to_string())?.ok()
    }
}

impl<'b, C> Decode<'b, C> for Recipient {
    fn decode(d: &mut Decoder<'b>, _: &mut C) -> Result<Self, decode::Error> {
        let position = d.position();
        Self::try_from_base58_string(d.str()?).map_err(|e| decode::Error::custom(e).at(position))
    }
}

pub mod offset_date_time {
    use minicbor::{Decoder, Encoder, decode as de, encode as en};
    use time::{OffsetDateTime, format_description::well_known::Rfc3339};

    pub fn encode<Ctx, W: en::Write>(
        v: &OffsetDateTime,
        e: &mut Encoder<W>,
        _: &mut Ctx,
    ) -> Result<(), en::Error<W::Error>> {
        let formatted = v.format(&Rfc3339).map_err(en::Error::custom)?;
        e.str(&formatted)?.ok()
    }

    pub fn decode<Ctx>(d: &mut Decoder<'_>, _: &mut Ctx) -> Result<OffsetDateTime, de::Error> {
        let position = d.position();
        OffsetDateTime::parse(d.str()?, &Rfc3339).map_err(|e| de::Error::custom(e).at(position))
    }
}

pub mod option_offset_date_time {
    use minicbor::{Decoder, Encoder, data::Type, decode as de, encode as en};
    use time::OffsetDateTime;

    use super::offset_date_time;

    pub fn encode<Ctx, W: en::Write>(
        v: &Option<OffsetDateTime>,
        e: &mut Encoder<W>,
        ctx: &mut Ctx,
    ) -> Result<(), en::Error<W::Error>> {
        match v {
            Some(v) => offset_date_time::encode(v, e, ctx),
            None => e.null()?.ok(),
        }
    }

    pub fn decode<Ctx>(
        d: &mut Decoder<'_>,
        ctx: &mut Ctx,
    ) -> Result<Option<OffsetDateTime>, de::Error> {
        if d.datatype()? == Type::Null {
            d.skip()?;
            Ok(None)
        } else {
            offset_date_time::decode(d, ctx).map(Some)
        }
    }

    pub fn nil() -> Option<Option<OffsetDateTime>> {
        Some(None)
    }

    pub fn is_nil(v: &Option<OffsetDateTime>) -> bool {
        v.is_none()
    }
}

pub mod option_arc {
    use std::sync::Arc;

    use minicbor::{Decode, Decoder, Encode, Encoder, decode as de, encode as en};

    pub fn encode<Ctx, T: Encode<Ctx>, W: en::Write>(
        v: &Option<Arc<T>>,
        e: &mut Encoder<W>,
        ctx: &mut Ctx,
    ) -> Result<(), en::Error<W::Error>> {
        v.as_deref().encode(e, ctx)
    }

    pub fn decode<'b, Ctx, T: Decode<'b, Ctx>>(
        d: &mut Decoder<'b>,
        ctx: &mut Ctx,
    ) -> Result<Option<Arc<T>>, de::Error> {
        Option::<T>::decode(d, ctx).map(|v| v.map(Arc::new))
    }

    pub fn nil<T>() -> Option<Option<Arc<T>>> {
        Some(None)
    }

    pub fn is_nil<T>(v: &Option<Arc<T>>) -> bool {
        v.is_none()
    }
}

pub mod option_json {
    use minicbor::{Decoder, Encoder, data::Type, decode as de, encode as en};
    use serde::{Serialize, de::DeserializeOwned};

    pub fn encode<Ctx, T: Serialize, W: en::Write>(
        v: &Option<T>,
        e: &mut Encoder<W>,
        _: &mut Ctx,
    ) -> Result<(), en::Error<W::Error>> {
        match v {
            Some(v) => {
                let json = serde_json::to_string(v).map_err(en::Error::custom)?;
                e.str(&json)?.ok()
            }
            None => e.null()?.ok(),
        }
    }

    pub fn decode<Ctx, T: DeserializeOwned>(
        d: &mut Decoder<'_>,
        _: &mut Ctx,
    ) -> Result<Option<T>, de::Error> {
        if d.datatype()? == Type::Null {
            d.skip()?;
            Ok(None)
        } else {
            let position = d.position();
            serde_json::from_str(d.str()?)
                .map(Some)
                .map_err(|e| de::Error::custom(e).at(position))
        }
    }

    pub fn nil<T>() -> Option<Option<T>> {
        Some(None)
    }

    pub fn is_nil<T>(v: &Option<T>) -> bool {
        v.is_none()
    }
}
