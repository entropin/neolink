***REMOVED*** Neolink Docker image build scripts
***REMOVED*** Copyright (c) 2020 George Hilliard
***REMOVED*** SPDX-License-Identifier: AGPL-3.0-only

FROM docker.io/rust:1-alpine AS build
MAINTAINER thirtythreeforty@gmail.com

RUN apk add --no-cache -X http://dl-cdn.alpinelinux.org/alpine/edge/testing \
  gst-rtsp-server-dev
RUN apk add --no-cache musl-dev gcc

***REMOVED*** Use static linking to work around https://github.com/rust-lang/rust/pull/58575
ENV RUSTFLAGS='-C target-feature=-crt-static'

***REMOVED*** Compile dependencies before main app to save rebuild time
***REMOVED*** https://github.com/errmac-v/cargo-build-dependencies
RUN cargo install cargo-build-dependencies
RUN mkdir /usr/local/src \
  && cd /usr/local/src \
  && USER=root cargo new --bin neolink
WORKDIR /usr/local/src/neolink
COPY Cargo.toml Cargo.lock ./
RUN cargo build-dependencies --release

***REMOVED*** Build the main program
COPY . /usr/local/src/neolink
RUN cargo build --release

***REMOVED*** Create the release container. Match the base OS used to build
FROM docker.io/alpine:latest

RUN apk add --no-cache -X http://dl-cdn.alpinelinux.org/alpine/edge/testing gst-rtsp-server
RUN apk add libgcc

COPY --from=build \
  /usr/local/src/neolink/target/release/neolink \
  /usr/local/bin/neolink
COPY docker/entrypoint.sh /entrypoint.sh

CMD ["/usr/local/bin/neolink", "--config", "/etc/neolink.toml"]
ENTRYPOINT ["/entrypoint.sh"]
EXPOSE 8554 
