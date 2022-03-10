FROM registry.access.redhat.com/ubi8/ubi-minimal:latest

LABEL org.opencontainers.image.source="https://github.com/drogue-iot/drogue-postgresql-pusher"

COPY target/release/drogue-postgresql-pusher /

ENTRYPOINT [ "/drogue-postgresql-pusher" ]
