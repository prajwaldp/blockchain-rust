pub mod traits {
    pub trait Hashable {
        fn encode(&self) -> Vec<u8>;

        fn hash(&self) -> Vec<u8> {
            crypto_hash::digest(crypto_hash::Algorithm::SHA256, &self.encode())
        }
    }
}

pub mod constants {
    pub const CHECKSUM_LENGTH: usize = 4;
    pub const VERSION: u8 = 0x00;
}

pub mod types {
    pub type Bytes = Vec<u8>;
}
