FROM rust:latest AS base

RUN apt-get update && apt-get install -y --no-install-recommends \
    # Bevy (x11)
    libx11-dev libxcursor-dev libxrandr-dev libxi-dev libxkbcommon-dev \
    libwayland-dev wayland-protocols \
    # Vulkan/OpenGL
    libvulkan-dev mesa-utils mesa-vulkan-drivers \
    # Áudio
    libasound2-dev libpulse-dev \
    # Input (gamepad)
    libudev-dev \
    # resvg (fontconfig)
    libfontconfig-dev \
    # Úteis
    pkg-config cmake gdb lldb \
    && rm -rf /var/lib/apt/lists/*

RUN rustup component add clippy rustfmt rust-analyzer

WORKDIR /workspace
