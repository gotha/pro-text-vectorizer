FROM rust:1-bookworm AS builder

WORKDIR /opt
RUN wget https://download.pytorch.org/libtorch/cu118/libtorch-cxx11-abi-shared-with-deps-2.1.0%2Bcu118.zip
RUN unzip libtorch-cxx11-abi-shared-with-deps-2.1.0+cu118.zip

ENV LIBTORCH=/opt/libtorch
ENV LD_LIBRARY_PATH=${LIBTORCH}/lib

WORKDIR /usr/src/app

COPY Cargo.toml Cargo.lock ./
COPY src ./src

RUN cargo build --release

FROM debian:bookworm

RUN apt-get update && apt-get install -y \
    ca-certificates \
    libgomp1

COPY --from=builder /usr/src/app/target/release/pro-text-vectorizer /usr/local/bin/pro-text-vectorizer
COPY --from=builder /opt/libtorch /opt/libtorch

ENV LIBTORCH=/opt/libtorch
ENV LD_LIBRARY_PATH=${LIBTORCH}/lib

CMD ["/usr/local/bin/pro-text-vectorizer"]
