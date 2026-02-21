FROM mcr.microsoft.com/devcontainers/base:ubuntu-22.04

# システムライブラリのインストール (Tauri & Rust & Hono 用)
RUN apt-get update && export DEBIAN_FRONTEND=noninteractive \
    && apt-get -y install --no-install-recommends \
    curl wget build-essential libssl-dev pkg-config \
    libgtk-3-dev libwebkit2gtk-4.1-dev libayatana-appindicator3-dev librsvg2-dev \
    git python3 ca-certificates

# pnpm のインストール
RUN sudo -u vscode corepack enable && sudo -u vscode corepack prepare pnpm@latest --activate

# Rust のインストール (vscode ユーザーとして)
USER vscode
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/home/vscode/.cargo/bin:${PATH}"

WORKDIR /devcontainer