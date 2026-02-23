FROM node:20-bookworm

ENV DEBIAN_FRONTEND=noninteractive

# Install system packages required for building the desktop app
RUN apt-get update && apt-get install -y \
    curl wget build-essential libssl-dev pkg-config \
    libgtk-3-dev libwebkit2gtk-4.1-dev libayatana-appindicator3-dev \
    librsvg2-dev git \
    && rm -rf /var/lib/apt/lists/*

# Install Rust toolchain (stable)
ENV RUSTUP_HOME=/usr/local/rustup \
    CARGO_HOME=/usr/local/cargo \
    PATH=/usr/local/cargo/bin:$PATH
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable --no-modify-path \
    && chmod -R a+w $RUSTUP_HOME $CARGO_HOME

# Install pnpm
RUN corepack enable && corepack prepare pnpm@10.12.1 --activate

WORKDIR /workspace