FROM rust:latest as builder

RUN rustup component add rustfmt --toolchain 1.47.0-x86_64-unknown-linux-gnu

COPY . /river

WORKDIR /river

RUN cargo build --release

FROM ubuntu:18.04

ENV LANG C.UTF-8

RUN apt update -y

# install gcc g++
RUN apt install -y gcc g++

# install python3.8
RUN apt install -y software-properties-common && \
    add-apt-repository -y ppa:deadsnakes/ppa && \
    apt install -y python3.8 python3-pip

# install rust
RUN apt install -y curl && \
    curl https://sh.rustup.rs -sSf | sh -s -- --default-toolchain stable -y
ENV PATH="/root/.cargo/bin:${PATH}"

# install node
RUN curl -sL https://deb.nodesource.com/setup_14.x | bash - && \
    apt install -y nodejs

# install go
RUN add-apt-repository -y ppa:longsleep/golang-backports && \
    apt install -y golang-go

# install openjdk
RUN apt install -y default-jdk

# TODO: install other languages
# TODO: C#
# TODO: Ruby
# TODO: PHP
# TODO: Lisp
# TODO: Kotlin
# TODO: Haskell

RUN rm -rf /var/lib/apt/lists/*

COPY ./plugins /plugins
ENV PATH="/plugins/js:${PATH}"
RUN /plugins/build.sh

WORKDIR /river

COPY --from=builder /river/target/release/river /river/

CMD [ "/river/river" ]
