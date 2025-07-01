FROM rust:slim

RUN apt-get update && apt-get install -y \
    build-essential \
    grub-pc-bin \
    qemu-system-i386 \
    nasm \
    llvm \
    lld \
    binutils \
    make \
    ca-certificates \
    xorriso \
    && rm -rf /var/lib/apt/lists/*

RUN rustup update \
    && rustup default nightly \
    && rustup component add rust-src --toolchain nightly-x86_64-unknown-linux-gnu

WORKDIR /kernel

COPY . .