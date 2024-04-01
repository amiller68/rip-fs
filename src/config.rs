use dotenvy::dotenv;
use std::env;

use url::Url;

#[derive(Debug)]
pub struct Config {
    sqlite_database_url: Url,
    ipfs_rpc_api_url: Url,
}

// TODO: arg parsing
impl Config {
    pub fn parse_env() -> Result<Config, ConfigError> {
        if dotenv().is_err() {
            tracing::warn!("No .env file found");
        }

        let sqlite_database_url_str = match env::var("SQLITE_DATABASE_URL") {
            Ok(url) => url,
            Err(_) => {
                tracing::warn!("No SQLITE_DATABASE_URL found in .env, using default");
                "sqlite://:memory:".to_string()
            }
        };
        let sqlite_database_url = Url::parse(&sqlite_database_url_str)?;

        let ipfs_rpc_api_url_str = match env::var("IPFS_RPC_API_URL") {
            Ok(url) => url,
            Err(_) => {
                tracing::warn!("No IPFS_RPC_API_URL found in .env, using default");
                "http://localhost:5001".to_string()
            }
        };
        let ipfs_rpc_api_url = Url::parse(&ipfs_rpc_api_url_str)?;

        Ok(Config {
            sqlite_database_url,
            ipfs_rpc_api_url,
        })
    }

    pub fn sqlite_database_url(&self) -> &Url {
        &self.sqlite_database_url
    }

    pub fn ipfs_rpc_api_url(&self) -> &Url {
        &self.ipfs_rpc_api_url
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("Invalid URL: {0}")]
    InvalidUrl(#[from] url::ParseError),
    #[error("Missing Env: {0}")]
    InvalidEnv(#[from] env::VarError),
}
