use crate::util::constants::*;
use crate::util::types::*;

use rand::rngs::OsRng;
use ripemd160::{Digest, Ripemd160};
use secp256k1::Secp256k1;
use serde::{Deserialize, Serialize};
use std::convert::TryInto;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

#[derive(Debug)]
pub struct Wallet {
    pub private_key: secp256k1::SecretKey,
    pub public_key: secp256k1::PublicKey,
    pub address: Bytes,
    pub public_key_hash: Bytes,
    pub checksum: Bytes,
    pub full_hash: Bytes,
}

// For serializing and deserializing wallets to and from files
#[derive(Serialize, Deserialize)]
pub struct WalletData {
    pub private_key: Bytes,
    pub public_key: Bytes,
    pub public_key_hash: Bytes,
}

impl Wallet {
    pub fn new() -> Self {
        // Use the Eliptical Curve Digital Signing Algorithm to create a wallet with a
        // public and private key
        let secp = Secp256k1::new();
        let mut rng = OsRng::new().expect("OsRng");

        // 1. The Private Key is randomly generated
        // 2. ECDSA is used to generate the Public Key from the Private Key
        let (secret_key, public_key) = secp.generate_keypair(&mut rng);

        let mut wallet = Wallet {
            private_key: secret_key,
            public_key: public_key,
            address: vec![],
            public_key_hash: vec![],
            checksum: vec![],
            full_hash: vec![],
        };

        wallet.set_public_key_hash();
        wallet.set_address();

        // Save a copy of the data to disk
        let wallet_data = WalletData {
            public_key: wallet.public_key.serialize().to_vec(),
            private_key: wallet.private_key[..].to_vec(),
            public_key_hash: wallet.public_key_hash.clone(),
        };

        let json = serde_json::to_string(&wallet_data).unwrap();
        let pathname = format!("./tmp/{}.json", hex::encode(&wallet.address));
        let path = Path::new(&pathname);
        let display = path.display();
        let mut file = match File::create(&path) {
            Err(why) => panic!("couldn't create {}: {}", display, why),
            Ok(file) => file,
        };

        file.write(json.as_bytes()).expect("Couldn't write file");

        wallet
    }

    fn set_public_key_hash(&mut self) {
        self.public_key_hash =
            Wallet::generate_sha256_ripemd160_hash(&self.public_key.serialize().to_vec());
    }

    fn set_address(&mut self) {
        let mut hash: Bytes = vec![VERSION]; // The first hex digit is the version
        hash.extend(self.public_key_hash.clone()); // The next 20 digits is the RIPEMD160 hash of the public key
        let checksum = Self::generate_checksum(&hash); // The next 32 digits is the SHA256 hash (twice) of the current hash
        hash.extend(&checksum);

        self.full_hash = hash.to_owned();
        self.checksum = checksum.to_vec();
        self.address = bs58::encode(&hash).into_vec();
    }

    fn generate_checksum(payload: &Vec<u8>) -> [u8; 4] {
        let first_hash = crypto_hash::digest(crypto_hash::Algorithm::SHA256, &payload);
        let second_hash = crypto_hash::digest(crypto_hash::Algorithm::SHA256, &first_hash);

        second_hash[..CHECKSUM_LENGTH]
            .try_into()
            .expect("expected slice to be 4 bytes long")
    }

    pub fn generate_sha256_ripemd160_hash(payload: &Bytes) -> Bytes {
        let public_key_hash = crypto_hash::digest(crypto_hash::Algorithm::SHA256, payload);

        let mut hasher = Ripemd160::new();
        hasher.input(public_key_hash);
        let hashed_result = hasher.result();
        hashed_result.to_vec()
    }

    pub fn is_address_valid(address: &Bytes) -> bool {
        let hash = bs58::decode(address).into_vec().unwrap();

        // Destructuring the components of the decoded address
        let version = hash[0];
        let public_key_hash = &hash[1..(hash.len() - CHECKSUM_LENGTH)];
        let actual_checksum = &hash[(hash.len() - CHECKSUM_LENGTH)..];

        let mut new_hash: Bytes = vec![version];
        new_hash.extend(public_key_hash);

        let target_checksum = Self::generate_checksum(&new_hash);

        actual_checksum == target_checksum
    }

    pub fn from_address(address: &Bytes) -> WalletData {
        let pathname = format!("./tmp/{}.json", hex::encode(address));
        let path = Path::new(&pathname);
        let mut file = File::open(path).expect("Couldn't open file");

        let mut s = String::new();
        file.read_to_string(&mut s).expect("Couldn't read file");

        let w: WalletData = serde_json::from_str(&s).expect("Couldn't parse string");
        w
    }
}

impl std::fmt::Display for Wallet {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "Public Key: {}\nPrivate Key: {}\nAddress: {},\nPublic Key Hash: {}\nFull Hash: {}, \nChecksum: {}",
            self.public_key,
            self.private_key,
            hex::encode(&self.address),
            hex::encode(&self.public_key_hash),
            hex::encode(&self.full_hash),
            hex::encode(&self.checksum),
        )
    }
}
