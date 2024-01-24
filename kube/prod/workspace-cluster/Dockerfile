FROM debian:bookworm as builder

RUN apt update
RUN apt install -y postgresql

FROM debian:bookworm-slim

COPY --from=builder /lib/x86_64-linux-gnu/libpq* /lib/x86_64-linux-gnu/
COPY --from=builder /lib/x86_64-linux-gnu/libssl* /lib/x86_64-linux-gnu/
COPY --from=builder /lib/x86_64-linux-gnu/libcrypto.so.3 /lib/x86_64-linux-gnu/
COPY --from=builder /lib/x86_64-linux-gnu/libgssapi_krb5.so.2 /lib/x86_64-linux-gnu/
COPY --from=builder /lib/x86_64-linux-gnu/libkrb5.so.3 /lib/x86_64-linux-gnu/
COPY --from=builder /lib/x86_64-linux-gnu/libk5crypto.so.3 /lib/x86_64-linux-gnu/
COPY --from=builder /lib/x86_64-linux-gnu/libkrb5support.so.0 /lib/x86_64-linux-gnu/
COPY --from=builder /lib/x86_64-linux-gnu/liblber-2.5.so.0 /lib/x86_64-linux-gnu/
COPY --from=builder /lib/x86_64-linux-gnu/libsasl2.so.2 /lib/x86_64-linux-gnu/
COPY --from=builder /lib/x86_64-linux-gnu/libkeyutils.so.1 /lib/x86_64-linux-gnu/
COPY --from=builder /lib/x86_64-linux-gnu/libldap-2.5.so.0 /lib/x86_64-linux-gnu/

COPY ./dagger /bin/dagger
