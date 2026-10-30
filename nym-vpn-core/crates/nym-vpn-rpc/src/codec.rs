// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::marker::PhantomData;

use bytes::{Buf, BufMut};
use minicbor::{Decode, Encode};
use tonic::{
    Status,
    codec::{Codec, DecodeBuf, Decoder, EncodeBuf, Encoder},
};

pub struct CborCodec<T, U>(PhantomData<(T, U)>);

impl<T, U> Default for CborCodec<T, U> {
    fn default() -> Self {
        Self(PhantomData)
    }
}

impl<T, U> Codec for CborCodec<T, U>
where
    T: Encode<()> + Send + 'static,
    U: for<'b> Decode<'b, ()> + Send + 'static,
{
    type Encode = T;
    type Decode = U;
    type Encoder = CborEncoder<T>;
    type Decoder = CborDecoder<U>;

    fn encoder(&mut self) -> Self::Encoder {
        CborEncoder(PhantomData)
    }

    fn decoder(&mut self) -> Self::Decoder {
        CborDecoder(PhantomData)
    }
}

pub struct CborEncoder<T>(PhantomData<T>);

impl<T: Encode<()>> Encoder for CborEncoder<T> {
    type Item = T;
    type Error = Status;

    fn encode(&mut self, item: Self::Item, dst: &mut EncodeBuf<'_>) -> Result<(), Self::Error> {
        minicbor::encode(&item, minicbor::encode::write::Writer::new(dst.writer()))
            .map_err(|e| Status::internal(format!("failed to encode message: {e}")))
    }
}

pub struct CborDecoder<U>(PhantomData<U>);

impl<U: for<'b> Decode<'b, ()>> Decoder for CborDecoder<U> {
    type Item = U;
    type Error = Status;

    fn decode(&mut self, src: &mut DecodeBuf<'_>) -> Result<Option<Self::Item>, Self::Error> {
        let bytes = src.copy_to_bytes(src.remaining());
        minicbor::decode(&bytes)
            .map(Some)
            .map_err(|e| Status::internal(format!("failed to decode message: {e}")))
    }
}
