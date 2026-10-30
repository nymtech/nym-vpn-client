// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

//! Declarative replacement for `tonic-build`: generates the server trait, the server and the
//! client for a gRPC service that uses [`crate::codec::CborCodec`].
//!
//! ```ignore
//! service! {
//!     package nym_vpn;
//!     service NymVpnService;
//!
//!     rpc Info(Empty) -> VpnServiceInfo;
//!     stream ListenToEvents(Empty) -> TunnelEvent;
//! }
//! ```
//!
//! `rpc` declares a unary method and `stream` a server-streaming method. Each one becomes a
//! snake_case method on the `NymVpnService` trait and on `NymVpnServiceClient`, served under
//! `/<package>.<service>/<Name>`. Streaming methods also add a `<Name>Stream` associated type to
//! the trait.

macro_rules! service {
    (
        package $package:ident;
        service $service:ident;

        $($kind:ident $name:ident($input:ty) -> $output:ty;)*
    ) => {
        pastey::paste! {
            pub const SERVICE_NAME: &str = concat!(stringify!($package), ".", stringify!($service));

            pub trait $service: ::std::marker::Send + ::std::marker::Sync + 'static {
                $(
                    crate::macros::trait_method!(
                        $kind [<$name:snake>] [<$name Stream>] ($input) -> $output
                    );
                )*
            }

            pub struct [<$service Server>]<T> {
                inner: ::std::sync::Arc<T>,
            }

            impl<T> [<$service Server>]<T> {
                pub fn new(inner: T) -> Self {
                    Self {
                        inner: ::std::sync::Arc::new(inner),
                    }
                }
            }

            impl<T> ::std::clone::Clone for [<$service Server>]<T> {
                fn clone(&self) -> Self {
                    Self {
                        inner: ::std::sync::Arc::clone(&self.inner),
                    }
                }
            }

            impl<T> ::tonic::server::NamedService for [<$service Server>]<T> {
                const NAME: &'static str = SERVICE_NAME;
            }

            impl<T, B> ::tonic::codegen::Service<::tonic::codegen::http::Request<B>>
                for [<$service Server>]<T>
            where
                T: $service,
                B: ::tonic::codegen::Body + ::std::marker::Send + 'static,
                B::Error: ::std::convert::Into<::tonic::codegen::StdError>
                    + ::std::marker::Send
                    + 'static,
            {
                type Response = ::tonic::codegen::http::Response<::tonic::body::Body>;
                type Error = ::std::convert::Infallible;
                type Future = ::tonic::codegen::BoxFuture<Self::Response, Self::Error>;

                fn poll_ready(
                    &mut self,
                    _cx: &mut ::std::task::Context<'_>,
                ) -> ::std::task::Poll<::std::result::Result<(), Self::Error>> {
                    ::std::task::Poll::Ready(Ok(()))
                }

                fn call(&mut self, req: ::tonic::codegen::http::Request<B>) -> Self::Future {
                    let inner = ::std::sync::Arc::clone(&self.inner);
                    match req.uri().path() {
                        $(
                            concat!(
                                "/", stringify!($package), ".", stringify!($service),
                                "/", stringify!($name)
                            ) => crate::macros::server_call!(
                                $kind $service inner req [<$name:snake>] [<$name Stream>]
                                ($input) -> $output
                            ),
                        )*
                        _ => ::std::boxed::Box::pin(async move {
                            let mut response =
                                ::tonic::codegen::http::Response::new(::tonic::body::Body::default());
                            let headers = response.headers_mut();
                            headers.insert(
                                ::tonic::Status::GRPC_STATUS,
                                (::tonic::Code::Unimplemented as i32).into(),
                            );
                            headers.insert(
                                ::tonic::codegen::http::header::CONTENT_TYPE,
                                ::tonic::metadata::GRPC_CONTENT_TYPE,
                            );
                            Ok(response)
                        }),
                    }
                }
            }

            #[derive(Debug, Clone)]
            pub struct [<$service Client>]<T> {
                inner: ::tonic::client::Grpc<T>,
            }

            impl [<$service Client>]<::tonic::transport::Channel> {
                pub async fn connect<D>(
                    dst: D,
                ) -> ::std::result::Result<Self, ::tonic::transport::Error>
                where
                    D: ::std::convert::TryInto<::tonic::transport::Endpoint>,
                    D::Error: ::std::convert::Into<::tonic::codegen::StdError>,
                {
                    let channel = ::tonic::transport::Endpoint::new(dst)?.connect().await?;
                    Ok(Self::new(channel))
                }
            }

            impl<T> [<$service Client>]<T>
            where
                T: ::tonic::client::GrpcService<::tonic::body::Body>,
                T::Error: ::std::convert::Into<::tonic::codegen::StdError>,
                T::ResponseBody: ::tonic::codegen::Body<Data = ::tonic::codegen::Bytes>
                    + ::std::marker::Send
                    + 'static,
                <T::ResponseBody as ::tonic::codegen::Body>::Error:
                    ::std::convert::Into<::tonic::codegen::StdError> + ::std::marker::Send,
            {
                pub fn new(inner: T) -> Self {
                    Self {
                        inner: ::tonic::client::Grpc::new(inner),
                    }
                }

                $(
                    crate::macros::client_method!(
                        $kind $package $service $name [<$name:snake>] ($input) -> $output
                    );
                )*
            }
        }
    };
}

macro_rules! trait_method {
    (rpc $method:ident $stream:ident ($input:ty) -> $output:ty) => {
        fn $method(
            &self,
            request: ::tonic::Request<$input>,
        ) -> impl ::std::future::Future<
            Output = ::std::result::Result<::tonic::Response<$output>, ::tonic::Status>,
        > + ::std::marker::Send;
    };
    (stream $method:ident $stream:ident ($input:ty) -> $output:ty) => {
        type $stream: ::tonic::codegen::tokio_stream::Stream<
                Item = ::std::result::Result<$output, ::tonic::Status>,
            > + ::std::marker::Send
            + 'static;

        fn $method(
            &self,
            request: ::tonic::Request<$input>,
        ) -> impl ::std::future::Future<
            Output = ::std::result::Result<::tonic::Response<Self::$stream>, ::tonic::Status>,
        > + ::std::marker::Send;
    };
}

macro_rules! server_call {
    (rpc $service:ident $inner:ident $req:ident $method:ident $stream:ident ($input:ty) -> $output:ty) => {{
        struct Method<T>(::std::sync::Arc<T>);

        impl<T: $service> ::tonic::server::UnaryService<$input> for Method<T> {
            type Response = $output;
            type Future = ::tonic::codegen::BoxFuture<::tonic::Response<$output>, ::tonic::Status>;

            fn call(&mut self, request: ::tonic::Request<$input>) -> Self::Future {
                let inner = ::std::sync::Arc::clone(&self.0);
                ::std::boxed::Box::pin(async move { inner.$method(request).await })
            }
        }

        ::std::boxed::Box::pin(async move {
            let mut grpc = ::tonic::server::Grpc::new(crate::codec::CborCodec::default());
            Ok(grpc.unary(Method($inner), $req).await)
        })
    }};
    (stream $service:ident $inner:ident $req:ident $method:ident $stream:ident ($input:ty) -> $output:ty) => {{
        struct Method<T>(::std::sync::Arc<T>);

        impl<T: $service> ::tonic::server::ServerStreamingService<$input> for Method<T> {
            type Response = $output;
            type ResponseStream = T::$stream;
            type Future = ::tonic::codegen::BoxFuture<
                ::tonic::Response<Self::ResponseStream>,
                ::tonic::Status,
            >;

            fn call(&mut self, request: ::tonic::Request<$input>) -> Self::Future {
                let inner = ::std::sync::Arc::clone(&self.0);
                ::std::boxed::Box::pin(async move { inner.$method(request).await })
            }
        }

        ::std::boxed::Box::pin(async move {
            let mut grpc = ::tonic::server::Grpc::new(crate::codec::CborCodec::default());
            Ok(grpc.server_streaming(Method($inner), $req).await)
        })
    }};
}

macro_rules! client_method {
    (rpc $package:ident $service:ident $name:ident $method:ident ($input:ty) -> $output:ty) => {
        pub async fn $method(
            &mut self,
            request: impl ::tonic::IntoRequest<$input>,
        ) -> ::std::result::Result<::tonic::Response<$output>, ::tonic::Status> {
            crate::macros::client_call!(
                self unary $package $service $name request
            )
        }
    };
    (stream $package:ident $service:ident $name:ident $method:ident ($input:ty) -> $output:ty) => {
        pub async fn $method(
            &mut self,
            request: impl ::tonic::IntoRequest<$input>,
        ) -> ::std::result::Result<
            ::tonic::Response<::tonic::codec::Streaming<$output>>,
            ::tonic::Status,
        > {
            crate::macros::client_call!(
                self server_streaming $package $service $name request
            )
        }
    };
}

macro_rules! client_call {
    ($self:ident $call:ident $package:ident $service:ident $name:ident $request:ident) => {{
        $self.inner.ready().await.map_err(|e| {
            ::tonic::Status::unknown(format!("Service was not ready: {}", e.into()))
        })?;
        let path = ::tonic::codegen::http::uri::PathAndQuery::from_static(concat!(
            "/",
            stringify!($package),
            ".",
            stringify!($service),
            "/",
            stringify!($name)
        ));
        let mut request = ::tonic::IntoRequest::into_request($request);
        request.extensions_mut().insert(::tonic::GrpcMethod::new(
            concat!(stringify!($package), ".", stringify!($service)),
            stringify!($name),
        ));
        $self
            .inner
            .$call(request, path, crate::codec::CborCodec::default())
            .await
    }};
}

pub(crate) use client_call;
pub(crate) use client_method;
pub(crate) use server_call;
pub(crate) use service;
pub(crate) use trait_method;
