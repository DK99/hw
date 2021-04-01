FROM ubuntu:21.04

ENV DEBIAN_FRONTEND=noninteractive

RUN apt-get update && \
    apt-get install -y libssl-dev openssl pkg-config curl clang && \
    rm -rf /var/lib/apt/lists/*

COPY . /src
WORKDIR /src

RUN chmod +x /src/wait-for-it.sh

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | bash -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

RUN RUST_BACKTRACE=1 cargo build --release --features "official-server"

CMD /src/wait-for-it.sh hedgewars-db:3306 && ./target/release/hedgewars-server