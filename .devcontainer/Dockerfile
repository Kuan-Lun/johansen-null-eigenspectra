FROM rust:1.88

RUN apt-get update && \
    apt-get upgrade -y && \
    apt-get install -y \
    git pkg-config \
    build-essential liblapack-dev libblas-dev gfortran \
    bc pigz pbzip2 xz-utils && \
    apt-get clean && \
    rm -rf /var/lib/apt/lists/*

# 安裝 Rust 組件（格式化工具等）
RUN rustup component add rustfmt clippy

# 建立工作目錄
WORKDIR /usr/src/app
COPY . .

# 確保容器在啟動時自動從遠端倉庫更新代碼
RUN git fetch origin && \
    git reset --hard origin/master

RUN cargo fmt && cargo clean