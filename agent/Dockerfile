FROM debian:jessie

ENV USER root

# install curl (needed to install rust)
RUN apt-get update && apt-get install -y curl gdb g++-multilib lib32stdc++6 libssl-dev libncurses5-dev

# install rust
RUN curl -sL https://static.rust-lang.org/dist/rust-nightly-x86_64-unknown-linux-gnu.tar.gz | tar xvz -C /tmp
RUN /tmp/rust-nightly-x86_64-unknown-linux-gnu/install.sh

# install cargo
RUN curl -sL https://static.rust-lang.org/cargo-dist/cargo-nightly-x86_64-unknown-linux-gnu.tar.gz | tar xvz -C /tmp
RUN /tmp/cargo-nightly-x86_64-unknown-linux-gnu/install.sh

RUN mkdir /src
WORKDIR /src

ADD Cargo.toml /src/Cargo.toml
RUN cargo install

ADD . /src

RUN cargo build --release

CMD ["./target/release/trubadur"]
