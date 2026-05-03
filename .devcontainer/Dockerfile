# See here for image contents: https://github.com/devcontainers/images/tree/main/src/rust

FROM mcr.microsoft.com/devcontainers/rust:1-1-bookworm

# Install Python and uv for lxml reference tests
RUN apt-get update && export DEBIAN_FRONTEND=noninteractive \
    && apt-get -y install --no-install-recommends python3-full

COPY --from=ghcr.io/astral-sh/uv:latest /uv /usr/local/bin/uv

COPY postCreateCommand.sh /tmp/postCreateCommand.sh
RUN chmod +x /tmp/postCreateCommand.sh

USER vscode

# Install nightly rust
RUN rustup toolchain install nightly

ENV PATH="${PATH}:/home/vscode/.local/bin"
