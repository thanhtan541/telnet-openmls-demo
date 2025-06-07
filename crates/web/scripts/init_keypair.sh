#!/usr/bin/env bash

set -x
set -eo pipefail
# generate a private key with the correct length
# While 2048 is the minimum key length supported by specifications such as JOSE,
# it is recommended that you use 3072. This gives you 128-bit security.
# This command also uses an exponent of 65537, which you’ve likely seen serialized as “AQAB”.
openssl genrsa -out private-key.pem 3072

# generate corresponding public key
openssl rsa -in private-key.pem -pubout -out public-key.pem

#The owner of the key pair makes the public key available to anyone, but keeps the private key secret.
#A certificate verifies that an entity is the owner of a particular public key.
if [[ -z "${NO_SELF_SIGN_CERT}" ]]
then
  # optional: create a self-signed certificate
  openssl req -new -x509 -key private-key.pem -out cert.pem -days 360
fi

if [[ -z "${NO_PFX_FORMAT}" ]]
then
  # optional: convert pem to pfx
  openssl pkcs12 -export -inkey private-key.pem -in cert.pem -out cert.pfx
fi

if [[ -z "${NO_DER_FORMAT}" ]]
then
  # optional: convert pem to der - need to use for jwt
  echo >&2 "Convert pem files to der"
  openssl rsa -in private-key.pem -outform DER -out private-key.der
  openssl rsa -pubin \
            -in public-key.pem \
            -inform PEM \
            -RSAPublicKey_out \
            -outform DER \
            -out public-key.der
fi

