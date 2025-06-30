FROM rust:1.88

# 安裝系統套件（LAPACK, BLAS 等科學運算函式庫）
RUN apt update && \
    apt install -y build-essential liblapack-dev libblas-dev gfortran

# 建立工作目錄
WORKDIR /usr/src/app
COPY . .

# 預先編譯依賴
RUN cargo build --release

CMD ["cargo", "run", "--release"]
