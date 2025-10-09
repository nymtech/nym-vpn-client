# syntax=docker/dockerfile:1
FROM debian:12-slim

# Allow overriding Rust & Go versions at build time
ARG RUST_VERSION=1.86.0
ARG GO_VERSION=1.24.0

ENV RUST_VERSION=${RUST_VERSION} \
    GOROOT=/usr/local/go \
    GOPATH=/go \
    PATH=/root/.cargo/bin:/usr/local/go/bin:/go/bin:$PATH

# 1) Install system deps for Rust builds + curl/git/wget
RUN apt-get update \
 && DEBIAN_FRONTEND=noninteractive \
    apt-get install -y --no-install-recommends \
      build-essential \
      ca-certificates \
      pkg-config \
      libdbus-1-dev \
      libssl-dev \
      libelf-dev \
      libmnl-dev \
      libnftnl-dev \
      libudev-dev \
      git \
      curl \
      wget \
 && rm -rf /var/lib/apt/lists/*

# 2) Install rustup + the requested Rust toolchain
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \
      | sh -s -- -y --default-toolchain "${RUST_VERSION}"

# 3) Download & install official Go
RUN wget -q https://go.dev/dl/go${GO_VERSION}.linux-amd64.tar.gz \
 && rm -rf /usr/local/go \
 && tar -C /usr/local -xzf go${GO_VERSION}.linux-amd64.tar.gz \
 && rm go${GO_VERSION}.linux-amd64.tar.gz

# 4) Clone & build wireguard-go
RUN git clone https://git.zx2c4.com/wireguard-go /wireguard-go \
 && cd /wireguard-go \
 && go build -o wireguard-go \
 && mv wireguard-go /usr/local/bin/ \
 && cd / && rm -rf /wireguard-go

# 5) Default workdir & entrypoint
WORKDIR /workspace

COPY ./wireguard /workspace/wireguard

# invoke the workspace‐level Makefile to build wireguard
RUN make build-wireguard

ENTRYPOINT ["cargo"]
