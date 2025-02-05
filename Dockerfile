# 使用官方 Rust 镜像作为基础镜像
FROM rust:1.67 as builder

# 设置工作目录
WORKDIR /app

# 将当前目录下的代码复制到容器中
COPY . .

# 安装依赖并构建项目
RUN cargo build --release

# 使用官方 Debian 镜像作为运行时镜像
FROM debian:bullseye-slim

# 安装需要的运行时依赖 (如果有)
RUN apt-get update && apt-get install -y \
    libssl-dev \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# 将构建的二进制文件从构建镜像中复制到运行时镜像
COPY --from=builder /app/target/release/biliup-rs /usr/local/bin/biliup-rs

# 设置容器启动命令
ENTRYPOINT ["/usr/local/bin/biliup-rs"]
