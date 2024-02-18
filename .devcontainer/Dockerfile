# See here for image contents: https://github.com/microsoft/vscode-dev-containers/tree/v0.187.0/containers/rust/.devcontainer/base.Dockerfile

FROM mcr.microsoft.com/devcontainers/rust:1-1-bullseye

# [Optional] Uncomment this section to install additional packages.
RUN apt-get update && export DEBIAN_FRONTEND=noninteractive \
    && apt-get -y install --no-install-recommends python3-lxml

USER vscode

# Install nightly rust
RUN rustup toolchain install nightly

# Install pip
ENV PATH="${PATH}:/home/vscode/.local/bin"
RUN curl https://bootstrap.pypa.io/get-pip.py -o /tmp/get-pip.py \
    && python3 /tmp/get-pip.py \
    && rm /tmp/get-pip.py
