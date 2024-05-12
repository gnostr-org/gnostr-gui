FROM rust:1.70-bullseye as builder

RUN apt-get update \
    && apt-get install --no-install-recommends -y \
       build-essential \
       ca-certificates \
       curl \
       libfontconfig1-dev \
       libssl-dev \
       pkg-config \
       sudo \
    && apt -y autoremove \
    && apt clean \
    && rm -rf /var/lib/apt/lists/*
RUN cargo install cargo-deb@^1


# Build gossip
COPY ./ gui/
WORKDIR gui
RUN mkdir -p /gui/artifacts
RUN cargo deb
RUN ls /gui/target/debian/

FROM scratch as custom-exporter
COPY --from=builder /gui/target/debian/gnostr-gui*_amd64.deb .
