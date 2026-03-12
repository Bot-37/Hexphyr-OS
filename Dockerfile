FROM rust:bookworm

ENV DEBIAN_FRONTEND=noninteractive

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    cpio \
    dosfstools \
    efitools \
    gdisk \
    make \
    mtools \
    openssl \
    ovmf \
    python3 \
    qemu-system-x86 \
    sbsigntool \
    uuid-runtime \
    xorriso \
    && rm -rf /var/lib/apt/lists/*

RUN rustup target add x86_64-unknown-none x86_64-unknown-uefi

WORKDIR /workspace
COPY . /workspace

CMD ["bash"]
