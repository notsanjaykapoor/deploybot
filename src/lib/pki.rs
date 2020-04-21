
use openssl::hash::MessageDigest;
use openssl::pkey::{PKey, Public};
use openssl::rsa::{Rsa};
use openssl::sign::{Verifier};
use slog::{error, info};
use std::fs;
use std::io::{Error, ErrorKind};

#[derive(Debug)]
pub struct PkiCheck {
    pub id: String,
}

impl PkiCheck {
    pub fn new(id: &str) -> PkiCheck {
        PkiCheck {
            id: id.to_string(),
        }
    }

    pub fn call(&self, plaintext_message: &str, crypto_signature: &str, logger: &slog::Logger) -> std::io::Result<()> {
        let pki_dir = dotenv::var("PKI_DIR_ANY").unwrap();

        for entry in fs::read_dir(pki_dir)? {
            let dir = entry?;
            let name = dir.path().to_str().unwrap().to_string();

            // normalize crypto_signature by removing newlines
            let crypto_signature_normalized = crypto_signature.replace("\n", "");

            match PkiRead::new().call(&name, plaintext_message, &crypto_signature_normalized, &logger) {
                Some(_) => {
                    // found a valid key
                    info!(logger, "pki_check_ok"; "key" => name, "id" => &self.id);

                    return Ok(())
                },
                None => {
                    // keep trying
                }
            };
        }

        error!(logger, "pki_check_error"; "id" => &self.id);

        Err(Error::new(ErrorKind::Other, "pki authorization error"))
    }
}

#[derive(Debug)]
pub struct PkiRead {}

impl PkiRead {
    pub fn new() -> PkiRead {
        PkiRead {}
    }

    pub fn call(&self, file_path: &str, plaintext_message: &str, crypto_signature: &str, logger: &slog::Logger) -> Option<i32> {
        // read key from file
        let pki_data = std::fs::read(file_path).unwrap();

        let public_rsa: Rsa<Public> = match Rsa::public_key_from_pem(&pki_data) {
            Ok(object) => {
                object
            },
            Err(e) => {
                error!(logger, "pki_rsa_key_pem_error: {}", e);

                return None
            },
        };

        self._verify(public_rsa, plaintext_message, crypto_signature)
    }

    fn _verify(&self, rsa_key: Rsa<Public>, plaintext_message: &str, crypto_signature: &str) -> Option<i32> {
        let pkey = PKey::from_rsa(rsa_key).unwrap();

        let mut verifier = Verifier::new(MessageDigest::sha256(), &pkey).unwrap();
        verifier.update(&plaintext_message.as_bytes()).unwrap();

        // base64 decode crypto_signature
        let crypto_signature_decoded = match base64::decode(crypto_signature) {
            Err(_) => {
                return None
            },
            Ok(msg) => {
                msg
            }
        };

        match verifier.verify(&crypto_signature_decoded).unwrap() {
            false => {
                return None
            },
            true => {
                return Some(0)
            }
        };
    }
}
