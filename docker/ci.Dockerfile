# CI専用: Ubuntu 22.04 ベース
FROM ubuntu:22.04

ENV DEBIAN_FRONTEND=noninteractive

# Tauriのビルドに必要なシステムライブラリ
RUN apt-get update && apt-get install -y \
    curl \
    wget \
    build-essential \
    libssl-dev \
    pkg-config \
    libgtk-3-dev \
    libwebkit2gtk-4.1-dev \
    libayatana-appindicator3-dev \
    librsvg2-dev \
    git \
    && rm -rf /var/lib/apt/lists/*

# 2. Rust のインストール (システム全体にインストール)
ENV RUSTUP_HOME=/usr/local/rustup \
    CARGO_HOME=/usr/local/cargo \
    PATH=/usr/local/cargo/bin:$PATH
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --no-modify-path

# 3. pnpm のインストール
RUN curl -fsSL https://get.pnpm.io/install.sh | env PNPM_VERSION=latest SHELL="$(which bash)" bash -
ENV PATH="/root/.local/share/pnpm:${PATH}"

WORKDIR /workspace