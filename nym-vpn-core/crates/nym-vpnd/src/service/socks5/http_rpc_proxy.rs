// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

//! HTTP RPC proxy server that forwards requests through the lazy SOCKS5 wrapper
//!
//! This module implements an HTTP proxy that accepts HTTP requests,
//! extracts the target URL from the '?p=' query parameter,
//! and forwards them through the lazy SOCKS5 wrapper.

use super::socks5_wrapper::LazySocks5Wrapper;
use bytes::Bytes;
use reqwest::Client;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};
use tokio_util::sync::CancellationToken;
use tracing::{debug, error, info};

/// HTTP RPC proxy server
pub struct HttpRpcProxy {
    /// Listen address (e.g., "127.0.0.1:8545")
    listen_address: String,
    /// Cancellation token for shutdown
    cancel_token: CancellationToken,
    /// Reqwest client configured with SOCKS5 proxy
    http_client: Option<Arc<Client>>,
}

/// Errors from the HTTP RPC proxy
#[derive(Debug, thiserror::Error)]
pub enum HttpRpcProxyError {
    #[error("Failed to bind to address {0}: {1}")]
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
            .timeout(Duration::from_secs(60))
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
                            let cancel_token = self.cancel_token.clone();

                            // Spawn a task to handle this connection
                            tokio::spawn(async move {
                                Self::handle_connection(stream, addr, http_client, cancel_token).await;
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

    /// Handle a single HTTP connection
    async fn handle_connection(
        mut stream: TcpStream,
        addr: SocketAddr,
        http_client: Arc<Client>,
        _cancel_token: CancellationToken,
    ) {
        // Parse incoming HTTP request
        let incoming_request = match Self::parse_incoming_request(&mut stream, &addr).await {
            Ok(req) => req,
            Err(e) => {
                error!("Failed to parse incoming request from {}: {}", addr, e);
                Self::send_error_response(&mut stream, 400, "Bad Request").await;
                return;
            }
        };

        // Extract the 'p' parameter (target URL)
        let target_url = match Self::extract_target_url(&incoming_request.path) {
            Ok(url) => url,
            Err(e) => {
                error!("Failed to extract target URL from {}: {}", addr, e);
                Self::send_error_response(&mut stream, 400, "Missing or invalid 'p' parameter")
                    .await;
                return;
            }
        };

        info!(
            "Proxying {} request from {} to: {}",
            incoming_request.method, addr, target_url
        );

        // Forward the request through SOCKS5
        match Self::forward_request(
            &http_client,
            &incoming_request.method,
            &target_url,
            incoming_request.headers,
            incoming_request.body,
        )
        .await
        {
            Ok(response_data) => {
                if let Err(e) = Self::send_response(&mut stream, response_data).await {
                    error!("Failed to send response to {}: {}", addr, e);
                }
            }
            Err(e) => {
                error!("Failed to forward request through SOCKS5: {}", e);
                Self::send_error_response(&mut stream, 502, "Bad Gateway").await;
            }
        }
    }

    /// Parse the incoming HTTP request from the TCP stream
    async fn parse_incoming_request(
        stream: &mut TcpStream,
        addr: &SocketAddr,
    ) -> Result<IncomingRequest, String> {
        let mut reader = BufReader::new(stream);
        let mut request_line = String::new();

        // Read the request line
        reader
            .read_line(&mut request_line)
            .await
            .map_err(|e| format!("Failed to read request line: {}", e))?;

        if request_line.is_empty() {
            return Err("Empty request".to_string());
        }

        // Parse request line (e.g., "GET /?p=https://example.com HTTP/1.1")
        let parts: Vec<&str> = request_line.trim().split_whitespace().collect();
        if parts.len() < 3 {
            return Err(format!("Malformed request line: {}", request_line));
        }

        let method = parts[0].to_string();
        let path = parts[1].to_string();

        // Parse headers
        let mut headers = Vec::new();
        let mut content_length = 0;
        loop {
            let mut header_line = String::new();
            reader
                .read_line(&mut header_line)
                .await
                .map_err(|e| format!("Failed to read header: {}", e))?;

            // Empty line indicates end of headers
            if header_line.trim().is_empty() {
                break;
            }

            // Parse header (e.g., "Content-Type: application/json")
            if let Some(colon_pos) = header_line.find(':') {
                let name = header_line[..colon_pos].trim().to_string();
                let value = header_line[colon_pos + 1..].trim().to_string();

                // Track content-length for body reading
                if name.eq_ignore_ascii_case("content-length") {
                    content_length = value.parse::<usize>().unwrap_or(0);
                }

                // Skip Host header as we'll set it based on target URL
                if !name.eq_ignore_ascii_case("host") {
                    headers.push((name, value));
                }
            }
        }

        // Read body if present
        let body = if content_length > 0 {
            let mut body_buf = vec![0u8; content_length];
            reader
                .read_exact(&mut body_buf)
                .await
                .map_err(|e| format!("Failed to read body: {}", e))?;
            Bytes::from(body_buf)
        } else {
            Bytes::new()
        };

        debug!(
            "Parsed HTTP RPC request from {}: {} {} with {} headers and {} byte body",
            addr,
            method,
            path,
            headers.len(),
            body.len()
        );

        Ok(IncomingRequest {
            method,
            path,
            headers,
            body,
        })
    }

    /// Extract the target URL from the 'p' query parameter
    fn extract_target_url(path: &str) -> Result<String, String> {
        // Find the query string
        let query = if let Some(pos) = path.find('?') {
            &path[pos + 1..]
        } else {
            return Err("Missing query string".to_string());
        };

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
        method: &str,
        target_url: &str,
        headers: Vec<(String, String)>,
        body: Bytes,
    ) -> Result<ResponseData, String> {
        // Build the request
        let mut request_builder = match method {
            "GET" => client.get(target_url),
            "POST" => client.post(target_url),
            "PUT" => client.put(target_url),
            "DELETE" => client.delete(target_url),
            "HEAD" => client.head(target_url),
            "PATCH" => client.patch(target_url),
            _ => {
                return Err(format!("Unsupported HTTP method: {}", method));
            }
        };

        // Add all headers from the original request
        for (name, value) in headers {
            request_builder = request_builder.header(name, value);
        }

        // Add body if present
        if !body.is_empty() {
            request_builder = request_builder.body(body);
        }

        // Send the request through SOCKS5
        let response = request_builder
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        let status = response.status();
        let response_headers: Vec<(String, String)> = response
            .headers()
            .iter()
            .map(|(name, value)| {
                (
                    name.as_str().to_string(),
                    value.to_str().unwrap_or("").to_string(),
                )
            })
            .collect();

        let body = response
            .bytes()
            .await
            .map_err(|e| format!("Failed to read response body: {}", e))?;

        Ok(ResponseData {
            status: status.as_u16(),
            reason: status.canonical_reason().unwrap_or("Unknown").to_string(),
            headers: response_headers,
            body,
        })
    }

    /// Send the HTTP response back to the client
    async fn send_response(stream: &mut TcpStream, response: ResponseData) -> Result<(), String> {
        // Build response status line
        let mut response_str = format!("HTTP/1.1 {} {}\r\n", response.status, response.reason);

        // Add all headers from the response
        for (name, value) in response.headers {
            response_str.push_str(&format!("{}: {}\r\n", name, value));
        }

        // End headers
        response_str.push_str("\r\n");

        // Write response headers
        stream
            .write_all(response_str.as_bytes())
            .await
            .map_err(|e| format!("Failed to write response headers: {}", e))?;

        // Write response body
        stream
            .write_all(&response.body)
            .await
            .map_err(|e| format!("Failed to write response body: {}", e))?;

        stream
            .flush()
            .await
            .map_err(|e| format!("Failed to flush stream: {}", e))?;

        debug!(
            "Successfully sent HTTP RPC response: {} bytes",
            response.body.len()
        );

        Ok(())
    }

    /// Send an HTTP error response
    async fn send_error_response(stream: &mut TcpStream, status: u16, reason: &str) {
        let response = format!(
            "HTTP/1.1 {} {}\r\nContent-Length: 0\r\nConnection: close\r\n\r\n",
            status, reason
        );
        let _ = stream.write_all(response.as_bytes()).await;
    }
}

/// Represents a parsed incoming HTTP request
struct IncomingRequest {
    method: String,
    path: String,
    headers: Vec<(String, String)>,
    body: Bytes,
}

/// Represents the response data to send back
struct ResponseData {
    status: u16,
    reason: String,
    headers: Vec<(String, String)>,
    body: Bytes,
}

impl Drop for HttpRpcProxy {
    fn drop(&mut self) {
        debug!("Dropping HTTP RPC proxy");
        self.cancel_token.cancel();
    }
}
