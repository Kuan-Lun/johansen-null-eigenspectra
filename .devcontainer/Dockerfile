FROM rust:1.88

# 安裝系統套件（LAPACK, BLAS 等科學運算函式庫）
RUN apt-get update && \
    apt-get install -y build-essential liblapack-dev libblas-dev gfortran pkg-config && \
    apt-get clean && \
    rm -rf /var/lib/apt/lists/*

# 安裝 Rust 組件（格式化工具等）
RUN rustup component add rustfmt clippy

# 建立工作目錄
WORKDIR /usr/src/app
COPY . .