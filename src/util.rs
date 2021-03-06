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
    // pub const DIFFICULTY: u128 = 0x0000ffffffffffffffffffffffffffff;
    pub const DIFFICULTY: u128 = 0x0fffffffffffffffffffffffffffffff; // during development
    pub const BLOCK_MEMORY_POOL_SIZE: usize = 2;
}

pub mod types {
    pub type Bytes = Vec<u8>;
}

pub mod helper_functions {
    use log::warn;

    pub fn handle_result<T, E: std::fmt::Display>(result: Result<T, E>, desc: &'static str) {
        match result {
            Ok(_) => (),
            Err(err) => warn!("[Error] {} responsed to with {}", desc, err),
        }
    }
}
