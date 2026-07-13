# Copyright 2025 Nym Technologies SA <contact@nymtech.net>
# SPDX-License-Identifier: GPL-3.0-only
#
# Debian 12 builder for reproducible builds of the test harness.
# Test environment is also Debian 12 (important because of dynamically linked libraries)

FROM debian:bookworm-slim

# system deps
RUN apt-get update && apt-get install -y \
    gcc make git curl pkg-config \
    libdbus-1-dev libmnl-dev libnftnl-dev \
    protobuf-compiler \
    libssl-dev \
    autoconf flex bison \
    && rm -rf /var/lib/apt/lists/*

# ideally should correspond to what's in workspace root
ARG RUST_VERSION=1.91.0
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \
    | sh -s -- -y --default-toolchain "${RUST_VERSION}" --target x86_64-unknown-linux-gnu
ENV PATH="/root/.cargo/bin:${PATH}"

# Go toolchain (version matches wireguard/libwg/go.mod)
RUN curl -fsSL https://go.dev/dl/go1.25.0.linux-amd64.tar.gz \
    | tar -C /usr/local -xz
ENV PATH="/usr/local/go/bin:${PATH}"

# static libpcap (needed by test-runner)
RUN git clone https://github.com/the-tcpdump-group/libpcap.git /tmp/libpcap \
    && cd /tmp/libpcap \
    && ./autogen.sh \
    && ./configure --enable-remote=yes --enable-dbus=no --enable-shared=no \
    && make \
    && make install \
    && rm -rf /tmp/libpcap

# environment for static OpenSSL linking
ENV OPENSSL_STATIC=1 \
    OPENSSL_LIB_DIR=/usr/lib/x86_64-linux-gnu \
    OPENSSL_INCLUDE_DIR=/usr/include/openssl \
    TEST_MANAGER_STATIC=1

WORKDIR /build
