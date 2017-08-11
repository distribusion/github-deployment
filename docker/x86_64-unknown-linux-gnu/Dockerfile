# NOTICE: taken from here https://github.com/japaric/cross/tree/master/docker
FROM ubuntu:12.04

RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    ca-certificates \
    cmake \
    gcc \
    libc6-dev \
    make \
    pkg-config

COPY docker/xargo.sh /
RUN bash /xargo.sh

COPY docker/cmake.sh /
RUN apt-get purge --auto-remove -y cmake && \
    bash /cmake.sh 2.8.11

COPY docker/openssl.sh /
RUN apt-get install -y --no-install-recommends \
    g++ \
    zlib1g-dev && \
    bash /openssl.sh linux-x86_64

ENV OPENSSL_DIR=/openssl \
    OPENSSL_INCLUDE_DIR=/openssl/include \
    OPENSSL_LIB_DIR=/openssl/lib

# PROJECT: compilation
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    curl

ENV PATH=/root/.cargo/bin:$PATH
RUN curl -k https://sh.rustup.rs > rustup-init.sh \
    && chmod +x rustup-init.sh && ./rustup-init.sh -y \
    && rustup update \
    && rustup default 1.19.0

RUN apt-get clean autoclean && \
    apt-get autoremove -y && \
    rm -rf /var/lib/apt/lists/* /tmp/* /var/tmp/*

WORKDIR /tmp
COPY docker/compilation.mk /tmp/Makefile
ENV BIN_NAME github-deployment

CMD [ "/bin/bash" ]
