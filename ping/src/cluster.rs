//! Cluster wrapper.

use std::str::FromStr;

#[derive(Clone)]
pub enum Cluster {
    Devnet,
    Testnet,
    MainnetBeta,
}

impl std::fmt::Display for Cluster {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Devnet => "devnet",
            Self::Testnet => "testnet",
            Self::MainnetBeta => "mainnet-beta",
        };
        write!(f, "{}", s)
    }
}

impl Cluster {
    pub fn url(&self) -> &str {
        match self {
            Self::Devnet => "https://api.devnet.solana.com",
            Self::Testnet => "https://api.testnet.solana.com",
            Self::MainnetBeta => "https://api.mainnet-beta.solana.com",
        }
    }
}

impl FromStr for Cluster {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "devnet" => Ok(Self::Devnet),
            "testnet" => Ok(Self::Testnet),
            "mainnet-beta" => Ok(Self::MainnetBeta),
            _ => Err(format!("Invalid cluster: {}", s)),
        }
    }
}
