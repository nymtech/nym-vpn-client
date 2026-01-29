//! Receives HTTP requests and sends them to lazy SOCKS5 proxy.

use super::lazy_socks5::LazySocks5;
use bytes::Bytes;
use http_body_util::{BodyExt, Full};
use hyper::{
    Method, Request, Response, StatusCode, body::Incoming, server::conn::http1, service::service_fn,
};
use hyper_util::rt::TokioIo;
use reqwest::Client;
use std::{convert::Infallible, error::Error, net::SocketAddr, sync::Arc, time::Duration};
use tokio::net::TcpListener;
use tokio_util::sync::CancellationToken;
use tracing::{debug, error, info};

/// Configuration for the HTTP RPC proxy
#[derive(Debug, Clone)]
pub struct HttpRpcConfig {
    /// Listen address ex: 127.0.0.1:8545
    listen_address: SocketAddr,
    /// Request timeout duration
    request_timeout: Duration,
}

/// Errors from the HTTP RPC proxy
#[derive(Debug, thiserror::Error)]
pub enum HttpRpcError {
    #[error("Failed to bind to {0}: {1}")]
    BindError(String, std::io::Error),

    #[error("Internal error: {0}")]
    Internal(String),
}

/// HTTP RPC proxy state
pub struct HttpRpc {
    /// Configuration
    config: HttpRpcConfig,
    /// HTTP client
    http_client: Option<Arc<Client>>,
    /// Cancellation token for shutdown
    cancel_token: CancellationToken,
}

impl HttpRpc {
    /// Create a new HTTP RPC proxy
    pub fn new(
        listen_address: SocketAddr,
        request_timeout: Duration,
        cancel_token: CancellationToken,
    ) -> Self {
        Self {
            config: HttpRpcConfig {
                listen_address,
                request_timeout,
            },
            cancel_token,
            http_client: None,
        }
    }

    /// Start the HTTP RPC proxy server
    pub async fn start(&mut self, lazy_socks5: Arc<LazySocks5>) -> Result<(), HttpRpcError> {
        let listen_address = self.config.listen_address.to_string();
        info!("Starting HTTP RPC proxy on {}", listen_address);

        // Get the SOCKS5 proxy public URL
        let socks5_url = format!("socks5h://{}", lazy_socks5.public_address());
        info!("Configuring HTTP client with SOCKS5 proxy: {}", socks5_url);

        // Create reqwest client configured with SOCKS5 proxy
        let proxy = reqwest::Proxy::all(&socks5_url)
            .map_err(|e| HttpRpcError::Internal(format!("Failed to create proxy: {}", e)))?;

        let http_client = Client::builder()
            .proxy(proxy)
            .timeout(self.config.request_timeout)
            .build()
            .map_err(|e| HttpRpcError::Internal(format!("Failed to build HTTP client: {}", e)))?;

        self.http_client = Some(Arc::new(http_client));

        // Bind TCP listener
        let listener = TcpListener::bind(&listen_address)
            .await
            .map_err(|e| HttpRpcError::BindError(listen_address.clone(), e))?;

        let local_addr = listener
            .local_addr()
            .map_err(|e| HttpRpcError::Internal(format!("Failed to get local address: {}", e)))?;

        info!("HTTP RPC proxy listening on {}", local_addr);

        // Accept connections loop
        loop {
            tokio::select! {
                result = listener.accept() => {
                    match result {
                        Ok((stream, addr)) => {
                            debug!("Accepted HTTP RPC connection from {}", addr);
                            let Some(http_client) = self.http_client.clone() else {
                                error!("HTTP client not initialized, cannot handle request");
                                continue;
                            };

                            // Spawn a task to handle this connection
                            tokio::spawn(async move {
                                let io = TokioIo::new(stream);

                                // Create a service function for this connection
                                let service = service_fn(move |req| {
                                    Self::handle_request(req, http_client.clone(), addr)
                                });

                                if let Err(e) = http1::Builder::new()
                                    .serve_connection(io, service)
                                    .await
                                {
                                    error!("Error serving connection from {}: {}", addr, e);
                                }
                            });
                        }
                        Err(e) => {
                            error!("Failed to accept HTTP RPC connection: {}", e);
                        }
                    }
                }
                _ = self.cancel_token.cancelled() => {
                    info!("HTTP RPC proxy shutting down");
                    break;
                }
            }
        }

        info!("HTTP RPC proxy server stopped");
        Ok(())
    }

    /// Handle a single HTTP request
    async fn handle_request(
        req: Request<Incoming>,
        http_client: Arc<Client>,
        client_addr: SocketAddr,
    ) -> Result<Response<Full<Bytes>>, Infallible> {
        let start_time = std::time::Instant::now();

        // Extract method and URI
        let method = req.method().clone();
        let uri = req.uri().clone();

        // Extract target URL from query parameter
        let target_url = match Self::extract_target_url(uri.query()) {
            Ok(url) => url,
            Err(e) => {
                error!("Failed to extract target URL from {}: {}", client_addr, e);
                return Ok(Self::error_response(
                    StatusCode::BAD_REQUEST,
                    "Missing or invalid 'p' parameter",
                ));
            }
        };

        // Log the incoming request
        info!(
            "Attempting to proxy {} request from {} to: {}",
            method, client_addr, target_url
        );

        // Collect request headers and body
        let (parts, body) = req.into_parts();
        let body_bytes = match body.collect().await {
            Ok(collected) => collected.to_bytes(),
            Err(e) => {
                error!("Failed to read request body from {}: {}", client_addr, e);
                return Ok(Self::error_response(
                    StatusCode::BAD_REQUEST,
                    "Failed to read request body",
                ));
            }
        };

        debug!(
            "Building {} request to {} with body size: {}",
            method,
            target_url,
            body_bytes.len()
        );

        // Forward the request through SOCKS5
        match Self::forward_request(
            &http_client,
            &method,
            &target_url,
            parts.headers,
            body_bytes,
        )
        .await
        {
            Ok(response_bytes) => {
                let elapsed = start_time.elapsed();
                info!(
                    "Successfully proxied {} request to {} in {:?}",
                    method, target_url, elapsed
                );
                Ok(response_bytes)
            }
            Err(e) => {
                let elapsed = start_time.elapsed();
                error!(
                    "Failed to forward {} request to {} after {:?}: {}",
                    method, target_url, elapsed, e
                );
                Ok(Self::error_response(StatusCode::BAD_GATEWAY, "Proxy error"))
            }
        }
    }

    /// Extract the target URL from the 'p' query parameter
    fn extract_target_url(query: Option<&str>) -> Result<String, String> {
        let query = query.ok_or("Missing query string")?;

        // Parse query parameters
        for param in query.split('&') {
            if let Some((key, value)) = param.split_once('=')
                && key == "p"
            {
                // URL decode the value
                let decoded = urlencoding::decode(value)
                    .map_err(|e| format!("Failed to decode URL: {}", e))?;
                return Ok(decoded.to_string());
            }
        }

        Err("Missing 'p' parameter in query string".to_string())
    }

    /// Forward the request through SOCKS5
    async fn forward_request(
        client: &Client,
        method: &Method,
        target_url: &str,
        headers: hyper::HeaderMap,
        body: Bytes,
    ) -> Result<Response<Full<Bytes>>, Box<dyn Error + Send + Sync>> {
        let send_start = std::time::Instant::now();

        // Build the reqwest request
        let mut request_builder = client.request(method.clone(), target_url);

        // Copy headers from hyper to reqwest, but skip host-specific headers
        // reqwest will set these correctly based on the target URL
        for (name, value) in headers.iter() {
            let name_str = name.as_str();
            // Skip headers that should be set by reqwest based on the target URL
            if name_str.eq_ignore_ascii_case("host")
                || name_str.eq_ignore_ascii_case("connection")
                || name_str.eq_ignore_ascii_case("content-length")
            {
                continue;
            }
            if let Ok(value_str) = value.to_str() {
                request_builder = request_builder.header(name_str, value_str);
            }
        }

        // Add body if present
        if !body.is_empty() {
            request_builder = request_builder.body(body.clone());
        }

        debug!(
            "Sending request through SOCKS5 to {} with {} headers",
            target_url,
            headers.len()
        );

        // Send the request through SOCKS5
        let response = request_builder.send().await.map_err(|e| {
            let send_elapsed = send_start.elapsed();
            error!("Request failed after {:?}: {}", send_elapsed, e);

            // Log specific error types
            if e.is_timeout() {
                error!(
                    "Request timeout for {} after {:?}",
                    target_url, send_elapsed
                );
            } else if e.is_connect() {
                error!(
                    "Connection error for {} after {:?}",
                    target_url, send_elapsed
                );
            }

            // Log error source chain
            if let Some(source) = e.source() {
                error!("Error source: {}", source);
            }

            e
        })?;

        let status = response.status();
        debug!("Received response from {}: status={}", target_url, status);

        // Build hyper response
        let mut hyper_response = Response::builder().status(status);

        // Copy headers from reqwest to hyper
        for (name, value) in response.headers().iter() {
            hyper_response = hyper_response.header(name, value);
        }

        // Read response body
        debug!("Reading response body from {}", target_url);
        let response_bytes = response.bytes().await.map_err(|e| {
            error!("Failed to read response body: {}", e);
            if let Some(source) = e.source() {
                error!("Error source when reading body: {}", source);
            }
            e
        })?;

        debug!(
            "Successfully read {} bytes from {}",
            response_bytes.len(),
            target_url
        );

        // Build final response
        let response_body = Full::new(response_bytes);
        Ok(hyper_response
            .body(response_body)
            .expect("Response builder should never fail with valid body"))
    }

    /// Create an error response
    fn error_response(status: StatusCode, message: &str) -> Response<Full<Bytes>> {
        Response::builder()
            .status(status)
            .body(Full::new(Bytes::from(message.to_string())))
            .expect("Response builder should never fail with valid status and body")
    }
}
