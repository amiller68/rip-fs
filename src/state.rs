use chromadb::v1::{client::ChromaClientOptions, ChromaClient};

use crate::config::Config;
use crate::database::Database;
use crate::engine::OllamaEngine;

pub struct State {
    sqlite_database: Database,
    chroma_database: ChromaClient,
    engine: OllamaEngine,
}

#[allow(dead_code)]
impl State {
    pub fn sqlite_database(&self) -> &Database {
        &self.sqlite_database
    }

    pub fn chroma_database(&self) -> &ChromaClient {
        &self.chroma_database
    }

    pub fn engine(&self) -> &OllamaEngine {
        &self.engine
    }

    pub async fn from_config(config: &Config) -> Result<Self, StateSetupError> {
        let sqlite_database = Database::connect(&config.sqlite_database_url()).await?;
        let chroma_database = ChromaClient::new(ChromaClientOptions {
            url: config.chroma_database_url().to_string(),
        });
        let engine = OllamaEngine::new(config.ollama_server_url());

        Ok(Self {
            sqlite_database,
            chroma_database,
            engine,
        })
    }
}

#[derive(Debug, thiserror::Error)]
pub enum StateSetupError {
    #[error("failed to setup the database: {0}")]
    DatabaseSetup(#[from] crate::database::DatabaseSetupError),
    #[error("failed to setup the Chroma database: {0}")]
    EngineSetup(#[from] crate::engine::OllamaEngineError),
}
