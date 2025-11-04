//! HTTP RPC proxy server that forwards requests through the SOCKS5 wrapper
//! Requests to http://localhost:8545/?p=TARGET_URL are forwarded to TARGET_URL
//! through the mixnet.

use super::socks5_wrapper::LazySocks5Wrapper;
use bytes::Bytes;
use http_body_util::{BodyExt, Full};
use hyper::body::Incoming;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Method, Request, Response, StatusCode};
use hyper_util::rt::TokioIo;
use reqwest::Client;
use std::convert::Infallible;
use std::error::Error;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::net::TcpListener;
use tokio_util::sync::CancellationToken;
use tracing::{debug, error, info};

/// HTTP RPC proxy server
pub struct HttpRpcProxy {
    /// Listen address ex: 127.0.0.1:8545
    listen_address: String,
    /// Cancellation token for shutdown
    cancel_token: CancellationToken,
    /// Reqwest client configured with SOCKS5 proxy
    http_client: Option<Arc<Client>>,
}

/// Errors from the HTTP RPC proxy
#[derive(Debug, thiserror::Error)]
pub enum HttpRpcProxyError {
    #[error("Failed to bind to {0}: {1}")]
    BindError(String, std::io::Error),

    #[error("Internal error: {0}")]
    Internal(String),
}

impl HttpRpcProxy {
    /// Create a new HTTP RPC proxy
    pub fn new(listen_address: String, cancel_token: CancellationToken) -> Self {
        Self {
            listen_address,
            cancel_token,
            http_client: None,
        }
    }

    /// Start the HTTP RPC proxy server
    pub async fn start(
        &mut self,
        socks5_wrapper: Arc<LazySocks5Wrapper>,
    ) -> Result<(), HttpRpcProxyError> {
        info!("Starting HTTP RPC proxy on {}", self.listen_address);

        // Get the SOCKS5 proxy URL from the wrapper's public address
        let socks5_url = format!("socks5h://{}", socks5_wrapper.public_address());
        info!("Configuring HTTP client with SOCKS5 proxy: {}", socks5_url);

        // Create reqwest client configured with SOCKS5 proxy
        let proxy = reqwest::Proxy::all(&socks5_url)
            .map_err(|e| HttpRpcProxyError::Internal(format!("Failed to create proxy: {}", e)))?;

        let http_client = Client::builder()
            .proxy(proxy)
            .timeout(Duration::from_secs(60)) // Total request timeout
            .connect_timeout(Duration::from_secs(30)) // Connection establishment timeout
            .pool_idle_timeout(Duration::from_secs(10)) // Very short idle timeout
            .pool_max_idle_per_host(0) // Disable connection pooling entirely
            .tcp_keepalive(Duration::from_secs(10)) // Aggressive TCP keepalive
            .build()
            .map_err(|e| {
                HttpRpcProxyError::Internal(format!("Failed to build HTTP client: {}", e))
            })?;

        self.http_client = Some(Arc::new(http_client));

        // Bind TCP listener
        let listener = TcpListener::bind(&self.listen_address)
            .await
            .map_err(|e| HttpRpcProxyError::BindError(self.listen_address.clone(), e))?;

        let local_addr = listener.local_addr().map_err(|e| {
            HttpRpcProxyError::Internal(format!("Failed to get local address: {}", e))
        })?;

        info!("HTTP RPC proxy listening on {}", local_addr);

        // Accept connections loop
        loop {
            tokio::select! {
                result = listener.accept() => {
                    match result {
                        Ok((stream, addr)) => {
                            debug!("Accepted HTTP RPC connection from {}", addr);
                            let http_client = self.http_client.clone().unwrap();

                            // Spawn a task to handle this connection
                            tokio::spawn(async move {
                                let io = TokioIo::new(stream);

                                // Create a service function for this connection
                                let service = service_fn(move |req| {
                                    Self::handle_request(req, http_client.clone(), addr)
                                });

                                // Serve HTTP/1 on this connection
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
            if let Some((key, value)) = param.split_once('=') {
                if key == "p" {
                    // URL decode the value
                    let decoded = urlencoding::decode(value)
                        .map_err(|e| format!("Failed to decode URL: {}", e))?;
                    return Ok(decoded.to_string());
                }
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

        // Copy headers from hyper to reqwest
        for (name, value) in headers.iter() {
            if let Ok(value_str) = value.to_str() {
                request_builder = request_builder.header(name.as_str(), value_str);
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
        Ok(hyper_response.body(response_body).unwrap())
    }

    /// Create an error response
    fn error_response(status: StatusCode, message: &str) -> Response<Full<Bytes>> {
        Response::builder()
            .status(status)
            .body(Full::new(Bytes::from(message.to_string())))
            .unwrap()
    }
}
