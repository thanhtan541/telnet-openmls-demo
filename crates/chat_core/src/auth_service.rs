// This module provides the functionality for the Authentication Service (AS).
// The AS is responsible for managing the authentication process and issuing tokens.

use serde::{Deserialize, Serialize};
use std::time::SystemTime;

#[derive(Debug, Serialize, Deserialize)]
enum CredentialType {
    Basic,
    X509,
}

#[derive(Debug, Serialize, Deserialize)]
enum SignatureScheme {
    Ed25519,
    EcdsaSecp256r1,
}

#[derive(Debug, Serialize, Deserialize)]
struct Credential {
    credential_type: CredentialType,
    identity: String,
    public_key: Vec<u8>,
    signature_scheme: SignatureScheme,
    valid_from: SystemTime,
    valid_until: Option<SystemTime>,
    revoked: bool,
}
