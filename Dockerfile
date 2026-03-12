FROM rustlang/rust:bookworm

ENV DEBIAN_FRONTEND=noninteractive

RUN apt-get update && apt-get install -y --no-install-recommends \
    grub-common \
    grub-pc-bin \
    qemu-system-x86 \
    xorriso \
    mtools \
    make \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

RUN rustup target add x86_64-unknown-none

WORKDIR /workspace
COPY . /workspace

CMD ["bash"]
