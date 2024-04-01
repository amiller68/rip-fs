use std::convert::TryFrom;

use crate::config::Config;
use crate::database::Database;
use crate::ipfs::IpfsRpcClient;

pub struct State {
    sqlite_database: Database,
    ipfs_rpc_client: IpfsRpcClient,
}

#[allow(dead_code)]
impl State {
    pub fn sqlite_database(&self) -> &Database {
        &self.sqlite_database
    }

    pub fn ipfs_rpc_client(&self) -> &IpfsRpcClient {
        &self.ipfs_rpc_client
    }

    pub async fn from_config(config: &Config) -> Result<Self, StateSetupError> {
        let sqlite_database = Database::connect(&config.sqlite_database_url()).await?;
        let ipfs_rpc_client = IpfsRpcClient::new(config.ipfs_rpc_api_url())?;

        Ok((Self {
            sqlite_database,
            ipfs_rpc_client,
        }))
    }
}

#[derive(Debug, thiserror::Error)]
pub enum StateSetupError {
    #[error("failed to setup the database: {0}")]
    DatabaseSetup(#[from] crate::database::DatabaseSetupError),
    #[error("failed to setup the IPFS RPC client: {0}")]
    IpfsRpcClientSetup(#[from] crate::ipfs::IpfsRpcClientError),
}
