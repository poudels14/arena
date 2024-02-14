FROM ubuntu:22.04 as builder

RUN apt update
RUN apt install -y libpython3.10-dev libfuse3-dev

# FROM ubuntu:22.04
FROM python:3.10.13-slim

COPY --from=builder /lib/x86_64-linux-gnu/libssl* /lib/x86_64-linux-gnu/
COPY --from=builder /lib/x86_64-linux-gnu/libcrypto.so.3 /lib/x86_64-linux-gnu/
COPY --from=builder /lib/x86_64-linux-gnu/libexpat* /lib/x86_64-linux-gnu/
COPY --from=builder /lib/x86_64-linux-gnu/libfuse3* /lib/x86_64-linux-gnu/
COPY --from=builder /lib/x86_64-linux-gnu/libpython3* /lib/x86_64-linux-gnu/

RUN pip install --no-cache-dir --upgrade pip && \
  pip install --no-cache-dir numpy pandas matplotlib faker
