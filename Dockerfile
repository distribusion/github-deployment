FROM japaric/x86_64-unknown-linux-gnu:latest

RUN apt-get update && apt-get install -y --no-install-recommends \
    build-essential ca-certificates make pkg-config curl

ENV PATH=/root/.cargo/bin:$PATH
RUN curl -k https://sh.rustup.rs > rustup-init.sh \
    && chmod +x rustup-init.sh && ./rustup-init.sh -y \
    && rustup update \
    && rustup default 1.19.0

RUN apt-get clean autoclean && \
    apt-get autoremove -y && \
    rm -rf /var/lib/apt/lists/* /tmp/* /var/tmp/*

WORKDIR /tmp

COPY src /tmp/src
COPY Cargo.toml /tmp/
COPY compilation.mk /tmp/Makefile

ENV BIN_NAME=github-deployment

CMD [ "/bin/bash" ]
