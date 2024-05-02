FROM debian:bookworm as builder

RUN apt update
RUN apt install -y postgresql

FROM debian:bookworm-slim

ARG ARCH

COPY ./arenasql-cluster-${ARCH} /bin/arenasql-cluster

ENTRYPOINT [ "/bin/bash", "-c", "/bin/arenasql-cluster init --catalogs-dir /var/data/catalogs --config /var/data/config.toml && RUST_BACKTRACE=1 /bin/arenasql-cluster start --config /var/data/config.toml --port 6000" ]
