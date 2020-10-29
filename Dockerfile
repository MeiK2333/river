FROM rust:latest as builder

COPY . /river

WORKDIR /river

RUN rustup component add rustfmt --toolchain 1.47.0-x86_64-unknown-linux-gnu && \
    cargo build --release


FROM ubuntu:18.04

ENV LANG C.UTF-8

RUN apt update

# install gcc g++
RUN apt install -y \
    g++ \
    gcc \
    libc6-dev \
    make \
    pkg-config

# install python3.6
RUN apt install -y software-properties-common && \
    add-apt-repository -y ppa:deadsnakes/ppa && \
    apt install python3.6

# install rust
RUN apt install -y curl && \
    curl https://sh.rustup.rs -sSf | sh -s -- --default-toolchain stable -y
ENV PATH="/root/.cargo/bin:${PATH}"

# install node
RUN curl -sL https://deb.nodesource.com/setup_14.x | bash - && \
    apt install -y nodejs

# install go
RUN add-apt-repository ppa:longsleep/golang-backports && \
    apt install golang-go

# TODO: install other languages

RUN rm -rf /var/lib/apt/lists/*

COPY ./plugins /plugins
ENV PATH="/plugins/js:${PATH}"
RUN /plugins/build.sh

WORKDIR /river

COPY --from=builder /river/target/release/river /river/

CMD [ "river" ]
