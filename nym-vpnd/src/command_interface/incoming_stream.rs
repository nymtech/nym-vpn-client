use std::path::Path;
use std::pin::Pin;
use std::task::Context;
use std::task::Poll;
use futures::TryStreamExt;
use tokio::io::AsyncRead;
use tokio::io::AsyncWrite;
use tokio::io::ReadBuf;
use tonic::transport::server::Connected;

#[derive(Debug)]
pub(super) struct StreamBox<T: AsyncRead + AsyncWrite>(pub T);

impl<T: AsyncRead + AsyncWrite> Connected for StreamBox<T> {
    type ConnectInfo = Option<()>;

    fn connect_info(&self) -> Self::ConnectInfo {
        None
    }
}
impl<T: AsyncRead + AsyncWrite + Unpin> AsyncRead for StreamBox<T> {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<std::io::Result<()>> {
        Pin::new(&mut self.0).poll_read(cx, buf)
    }
}
impl<T: AsyncRead + AsyncWrite + Unpin> AsyncWrite for StreamBox<T> {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<std::io::Result<usize>> {
        Pin::new(&mut self.0).poll_write(cx, buf)
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        Pin::new(&mut self.0).poll_flush(cx)
    }

    fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        Pin::new(&mut self.0).poll_shutdown(cx)
    }
}

pub(super) fn setup_incoming(
    socket_path: &Path,
) -> impl futures::Stream<Item = Result<impl AsyncRead + AsyncWrite, std::io::Error>> {
    let mut endpoint = parity_tokio_ipc::Endpoint::new(socket_path.to_string_lossy().to_string());
    endpoint.set_security_attributes(
        parity_tokio_ipc::SecurityAttributes::allow_everyone_create()
            .unwrap()
            .set_mode(0o766)
            .unwrap(),
    );
    endpoint.incoming().unwrap()
}

pub(super) fn setup_incoming_stream(
    socket_path: &Path,
) -> impl futures::Stream<Item = Result<StreamBox<impl AsyncRead + AsyncWrite>, std::io::Error>> {
    setup_incoming(socket_path).map_ok(StreamBox)
}
