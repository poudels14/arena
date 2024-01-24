FROM debian:bookworm as builder

RUN apt update
RUN apt install -y postgresql

FROM debian:bookworm-slim

COPY ./arenasql-cluster /bin/arenasql-cluster

ENTRYPOINT [ "/bin/bash", "-c", "/bin/arenasql-cluster init --dir /var/data/ && /bin/arenasql-cluster start --root /var/data/ --port 6000" ]
