#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all(deserialize = "lowercase", serialize = "lowercase"))]
pub enum Hash {
    SHA256(String),
    MD5(String),
}

use crate::releases::ChecksumError;
use serde::{Deserialize, Serialize};
use sha2::Digest;

impl Hash {
    pub fn hash_type(&self) -> &'static str {
        match self {
            Hash::SHA256(_) => "SHA-256",
            Hash::MD5(_) => "MD5",
        }
    }
    pub fn verify(&self, mut data: impl std::io::Read) -> Result<(), ChecksumError> {
        let (checksum, hash) = match self {
            Hash::SHA256(checksum) => {
                let mut sha256 = sha2::Sha256::new();
                std::io::copy(&mut data, &mut sha256)?;
                let hash = sha256.finalize();
                (checksum, format!("{:x}", hash))
            }
            Hash::MD5(checksum) => {
                let mut md5 = md5::Context::new();
                std::io::copy(&mut data, &mut md5)?;
                let hash = md5.compute();
                (checksum, format!("{:x}", hash))
            }
        };
        if checksum == &hash {
            Ok(())
        } else {
            Err(ChecksumError::InvalidChecksum {
                expected: checksum.clone(),
                got: hash,
            })
        }
    }
}
