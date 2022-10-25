FROM ubuntu:latest

ARG VER

RUN apt-get update \
    && apt-get install -qqy \
        wget gcc build-essential git \
    && apt-get clean autoclean \
    && apt-get autoremove -y --purge \
    && rm -rf /var/lib/apt/lists/* /tmp/* /var/tmp/* /var/lib/{apt,cache,log}

RUN set -eux; \
    wget -O rustup-init "https://sh.rustup.rs"; \
    chmod +x rustup-init; \
    ./rustup-init -y --no-modify-path --profile minimal --default-toolchain ${VER:-1.45.0}; \
    rm rustup-init;

ENV PATH="${PATH}:/root/.cargo/bin"

CMD \
    rustup --version; \
    cargo --version; \
    rustc --version;
