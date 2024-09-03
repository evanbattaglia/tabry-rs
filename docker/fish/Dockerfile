FROM ubuntu

ARG DEBIAN_FRONTEND=noninteractive

RUN apt-get update --quiet && \
    apt-get install --quiet --yes software-properties-common openssh-client git && \
    add-apt-repository --yes ppa:fish-shell/release-3 && \
    apt-get install --quiet --yes fish curl build-essential

SHELL ["fish", "--command"]

RUN chsh -s /usr/bin/fish

ENV SHELL /usr/bin/fish
ENV LANG=C.UTF-8 LANGUAGE=C.UTF-8 LC_ALL=C.UTF-8

ENTRYPOINT [ "fish" ]

# install rust
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs >> rustup.sh
RUN chmod +x rustup.sh
RUN ./rustup.sh -y

# install tabry
ADD ./fixtures /app/fixtures
ADD ./shell /app/shell
ADD ./src /app/src
ADD ./Cargo.toml /app/Cargo.toml
ADD ./Cargo.lock /app/Cargo.lock
ADD ./build.rs /app/build.rs
WORKDIR /app
RUN cargo install --path .

# setup tabry with foo example
ADD ./docker/fish/foo.tabry /tabry/foo.tabry