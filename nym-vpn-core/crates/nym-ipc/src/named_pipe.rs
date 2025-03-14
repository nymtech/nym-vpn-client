// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{
    ffi::{OsStr, OsString},
    io,
    pin::Pin,
    task::{Context, Poll},
};

use async_stream::try_stream;
use hyper_util::rt::TokioIo;
use tokio::{
    io::{AsyncRead, AsyncWrite, ReadBuf},
    net::windows::named_pipe::{ClientOptions, NamedPipeClient, NamedPipeServer, ServerOptions},
    time::{Duration, Instant},
};
use tokio_stream::Stream;
use tonic::transport::server::Connected;
use windows::Win32::Foundation;

use nym_windows::security::SecurityAttributes;

/// Connect timeout used when the pipe reports that it's busy.
const PIPE_AVAILABILITY_TIMEOUT: Duration = Duration::from_secs(5);

pub async fn connect(pipe_name: impl AsRef<OsStr>) -> io::Result<TokioIo<NamedPipeClient>> {
    let attempt_start = Instant::now();
    loop {
        match ClientOptions::new().read(true).write(true).open(&pipe_name) {
            Err(e) if e.raw_os_error() == Some(Foundation::ERROR_PIPE_BUSY.0 as i32) => {
                if attempt_start.elapsed() < PIPE_AVAILABILITY_TIMEOUT {
                    tokio::time::sleep(Duration::from_millis(50)).await;
                    continue;
                } else {
                    return Err(e);
                }
            }
            result => return result.map(TokioIo::new),
        }
    }
}

pub fn incoming(
    pipe_name: OsString,
) -> io::Result<impl Stream<Item = io::Result<Connector<NamedPipeServer>>>> {
    let permissions = Foundation::GENERIC_READ | Foundation::GENERIC_WRITE;
    let security_attributes = SecurityAttributes::allow_everyone(permissions.0)?;

    NamedPipeListener::new(pipe_name, security_attributes).incoming()
}

struct NamedPipeListener {
    pipe_name: OsString,
    created_listener: bool,
    security_attributes: SecurityAttributes,
}

impl NamedPipeListener {
    fn new(pipe_name: OsString, security_attributes: SecurityAttributes) -> Self {
        NamedPipeListener {
            pipe_name,
            created_listener: false,
            security_attributes,
        }
    }

    fn incoming(
        mut self,
    ) -> io::Result<impl Stream<Item = io::Result<Connector<NamedPipeServer>>>> {
        let mut listener = self.create_listener()?;
        Ok(try_stream! {
            loop {
                listener.connect().await?;

                let connected = listener;
                listener = self.create_listener()?;

                yield Connector(connected);
            }
        })
    }

    fn create_listener(&mut self) -> io::Result<NamedPipeServer> {
        let server = unsafe {
            ServerOptions::new()
                .first_pipe_instance(!self.created_listener)
                .reject_remote_clients(true)
                .access_inbound(true)
                .access_outbound(true)
                .in_buffer_size(u16::MAX as u32)
                .out_buffer_size(u16::MAX as u32)
                .create_with_security_attributes_raw(
                    &self.pipe_name,
                    self.security_attributes.as_mut_ptr() as _,
                )?
        };

        self.created_listener = true;

        Ok(server)
    }
}

#[derive(Debug)]
pub struct Connector<T: AsyncRead + AsyncWrite>(pub T);

impl<T: AsyncRead + AsyncWrite> Connected for Connector<T> {
    type ConnectInfo = Option<()>;

    fn connect_info(&self) -> Self::ConnectInfo {
        None
    }
}

impl<T: AsyncRead + AsyncWrite + Unpin> AsyncRead for Connector<T> {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        Pin::new(&mut self.0).poll_read(cx, buf)
    }
}

impl<T: AsyncRead + AsyncWrite + Unpin> AsyncWrite for Connector<T> {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        Pin::new(&mut self.0).poll_write(cx, buf)
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Pin::new(&mut self.0).poll_flush(cx)
    }

    fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Pin::new(&mut self.0).poll_shutdown(cx)
    }
}
