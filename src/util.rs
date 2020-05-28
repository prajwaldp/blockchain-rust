pub mod traits {
    pub trait Hashable {
        fn encode(&self) -> Vec<u8>;

        fn hash(&self) -> Vec<u8> {
            crypto_hash::digest(crypto_hash::Algorithm::SHA256, &self.encode())
        }
    }
}
