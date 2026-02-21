FROM mcr.microsoft.com/devcontainers/base:ubuntu-22.04

# 1. システムライブラリのインストール
RUN apt-get update && export DEBIAN_FRONTEND=noninteractive \
    && apt-get -y install --no-install-recommends \
    curl wget build-essential libssl-dev pkg-config \
    libgtk-3-dev libwebkit2gtk-4.1-dev libayatana-appindicator3-dev librsvg2-dev \
    git python3 ca-certificates \
    fonts-noto-cjk

# 2. pnpm のインストール (公式シェルスクリプトを使用)
# これにより、corepack の有無を問わず確実に /usr/local/bin に入ります
RUN curl -fsSL https://get.pnpm.io/install.sh | SHELL="$(which bash)" bash -

# 3. Rust のインストール (vscode ユーザー環境)
USER vscode
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/home/vscode/.cargo/bin:/root/.local/share/pnpm:${PATH}"

WORKDIR /devcontainer