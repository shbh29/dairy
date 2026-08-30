use aes_gcm::aead::{Aead, KeyInit};
use aes_gcm::{Aes256Gcm, Nonce};
use pbkdf2::pbkdf2_hmac;
use rand::RngCore;
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

trait Zeroize {
    fn zeroize(&mut self);
}

impl Zeroize for [u8] {
    fn zeroize(&mut self) {
        for byte in self.iter_mut() {
            *byte = 0;
        }
    }
}

impl Zeroize for String {
    fn zeroize(&mut self) {
        let bytes = unsafe { self.as_mut_vec() };
        bytes.zeroize();
    }
}

const PBKDF2_ITERATIONS: u32 = 100_000;

#[derive(Debug, Serialize, Deserialize)]
struct KeystoreFile {
    version: u32,
    salt: String,
    nonce: String,
    ciphertext: String,
    iterations: u32,
}

pub fn keystore_path() -> PathBuf {
    let path = Path::new("data").join("keystore.p12");
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    path
}

fn prompt_for_password() -> String {
    print!("Enter keystore password: ");
    let _ = io::stdout().flush();

    let mut password = String::new();
    io::stdin()
        .read_line(&mut password)
        .expect("failed to read keystore password");

    let password = password.trim_end_matches(&['\r', '\n'][..]).to_string();
    password
}

fn derive_key(passphrase: &str, salt: &[u8]) -> [u8; 32] {
    let mut key = [0u8; 32];
    pbkdf2_hmac::<Sha256>(passphrase.as_bytes(), salt, PBKDF2_ITERATIONS, &mut key);
    key
}

pub fn ensure_keystore_key(path: &Path) -> [u8; 32] {
    if let Ok(existing) = fs::read(path) {
        if !existing.is_empty() {
            let keystore: KeystoreFile = serde_json::from_slice(&existing)
                .expect("keystore file is not valid JSON");

            let mut password = prompt_for_password();
            let mut salt = hex::decode(&keystore.salt).expect("keystore salt is invalid hex");
            let mut nonce = hex::decode(&keystore.nonce).expect("keystore nonce is invalid hex");
            let mut ciphertext = hex::decode(&keystore.ciphertext)
                .expect("keystore ciphertext is invalid hex");

            let mut derived_key = derive_key(&password, &salt);
            let cipher = Aes256Gcm::new_from_slice(&derived_key).expect("32-byte AES key");

            let mut plaintext = cipher
                .decrypt(Nonce::from_slice(&nonce), ciphertext.as_ref())
                .expect("keystore password is incorrect or keystore is corrupted");

            let mut key = [0u8; 32];
            key.copy_from_slice(&plaintext);

            plaintext.zeroize();
            ciphertext.zeroize();
            nonce.zeroize();
            salt.zeroize();
            derived_key.zeroize();
            password.zeroize();

            return key;
        }
    }

    let mut password = prompt_for_password();
    let mut salt = [0u8; 16];
    rand::thread_rng().fill_bytes(&mut salt);

    let key = rand::random::<[u8; 32]>();
    let mut derived_key = derive_key(&password, &salt);
    let cipher = Aes256Gcm::new_from_slice(&derived_key).expect("32-byte AES key");
    let mut nonce = rand::random::<[u8; 12]>();
    let mut ciphertext = cipher
        .encrypt(Nonce::from_slice(&nonce), key.as_ref())
        .expect("encryption failed while creating keystore");

    let keystore = KeystoreFile {
        version: 1,
        salt: hex::encode(salt),
        nonce: hex::encode(nonce),
        ciphertext: hex::encode(ciphertext.clone()),
        iterations: PBKDF2_ITERATIONS,
    };

    fs::write(path, serde_json::to_string_pretty(&keystore).expect("serialize keystore"))
        .expect("Unable to write the password-protected keystore");

    ciphertext.zeroize();
    nonce.zeroize();
    salt.zeroize();
    derived_key.zeroize();
    password.zeroize();

    key
}
