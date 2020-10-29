FROM rust:latest as builder

COPY . /river

WORKDIR /river

RUN rustup component add rustfmt --toolchain 1.47.0-x86_64-unknown-linux-gnu && \
    cargo build --release

FROM ubuntu:18.04

ENV LANG C.UTF-8

# install python3.6
RUN apt update && \
    apt install -y software-properties-common && \
    add-apt-repository -y ppa:deadsnakes/ppa && \
    apt install python3.6

# install rust
RUN apt install -y curl && \
    curl https://sh.rustup.rs -sSf | sh -s -- --default-toolchain stable -y
ENV PATH=/root/.cargo/bin:$PATH

# TODO: install other languages

COPY ./plugins /plugins

RUN /plugins/build.sh

WORKDIR /river

COPY --from=builder /river/target/release/river /river/

CMD [ "river" ]
