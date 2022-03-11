FROM registry.access.redhat.com/ubi8/ubi:latest as builder

RUN dnf -y install openssl openssl-devel gcc gcc-c++ make libpq-devel cmake perl xz

ENV RUSTUP_HOME=/opt/rust
ENV CARGO_HOME=/opt/rust

RUN curl https://sh.rustup.rs -sSf | sh -s -- -y

ENV PATH "$PATH:$CARGO_HOME/bin"

RUN mkdir -p /usr/src/drogue-postgresql-pusher
ADD . /usr/src/drogue-postgresql-pusher

WORKDIR /usr/src/drogue-postgresql-pusher

RUN cargo build --release

FROM registry.access.redhat.com/ubi8/ubi-minimal:latest

LABEL org.opencontainers.image.source="https://github.com/drogue-iot/drogue-postgresql-pusher"

COPY --from=builder /usr/src/drogue-postgresql-pusher/target/release/drogue-postgresql-pusher /

ENTRYPOINT [ "/drogue-postgresql-pusher" ]
