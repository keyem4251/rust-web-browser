FROM --platform=linux/amd64 ubuntu:22.04

WORKDIR /workspace

RUN apt-get update
RUN apt-get install -y curl \
    qemu-system \
    build-essential \
    && apt-get clean && rm -rf /var/lib/apt/lists/*
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
