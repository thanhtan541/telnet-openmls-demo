#!/usr/bin/env bash

set -x
set -eo pipefail

openssl genpkey -algorithm RSA \
    -pkeyopt rsa_keygen_bits:2048 \
    -pkeyopt rsa_keygen_pubexp:65537 | \
  openssl pkcs8 -topk8 -nocrypt -outform der > private-key.pk8
