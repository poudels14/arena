#!/bin/bash

cd /opt/letsencrypt
certbot renew --pre-hook "service haproxy stop" --post-hook "service haproxy start"
DOMAIN='sidecar.so' sudo -E bash -c 'cat /etc/letsencrypt/live/$DOMAIN/fullchain.pem /etc/letsencrypt/live/$DOMAIN/privkey.pem > /etc/haproxy/certs/$DOMAIN.pem'
