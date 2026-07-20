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
    # Build rápido (linker mold + cache sccache)
    mold clang sccache \
    # Úteis
    pkg-config cmake gdb lldb \
    && rm -rf /var/lib/apt/lists/*

RUN rustup component add clippy rustfmt rust-analyzer

RUN cargo install cargo-ndk

# Ativa sccache como wrapper do rustc via ENV (decisão do Arquiteto, ADR-009):
# env var LOCAL do container — nunca no .cargo/config.toml versionado, que
# quebraria o CI (runners do GitHub não têm sccache).
ENV RUSTC_WRAPPER=sccache

# OpenCode CLI
RUN curl -fsSL https://opencode.ai/install | bash && ln -sf /root/.opencode/bin/opencode /usr/local/bin/opencode

WORKDIR /workspace
