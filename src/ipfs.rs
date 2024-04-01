use std::io::Read;
use std::ops::Deref;
use std::str::FromStr;

use async_trait::async_trait;
use banyanfs::stores::{DataStore, DataStoreError};
use futures_util::TryStreamExt;
use http::uri::Scheme;
use ipfs_api_backend_hyper::request::{Add as AddRequest, BlockPut as BlockPutRequest};
use ipfs_api_backend_hyper::IpfsApi;
use ipfs_api_backend_hyper::{IpfsClient, TryFromUri};
use url::Url;

use crate::types::{BanyanCid, IpldCodec, MhCode, RipCid};

/* Constants */

const DEFAULT_CID_VERSION: u32 = 1;
const DEFAULT_MH_TYPE: &str = "blake3";

/* Ipfs IpfsRpcClient Client Wrapper */

#[derive(Clone)]
pub struct IpfsRpcClient(IpfsClient);

impl Default for IpfsRpcClient {
    fn default() -> Self {
        let url: Url = "http://localhost:5001".try_into().unwrap();
        Self::new(&url).unwrap()
    }
}

impl Deref for IpfsRpcClient {
    type Target = IpfsClient;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[async_trait(?Send)]
impl DataStore for IpfsRpcClient {
    async fn contains_cid(&self, cid: BanyanCid) -> Result<bool, DataStoreError> {
        let rip_cid = RipCid::from(cid.clone());
        self.has_block(&rip_cid).await.map_err(|e| e.into())
    }

    async fn remove(&mut self, cid: BanyanCid, recusrive: bool) -> Result<(), DataStoreError> {
        let rip_cid = RipCid::from(cid.clone());
        self.rm_block(&rip_cid, recusrive)
            .await
            .map_err(|e| e.into())
    }

    async fn retrieve(&self, cid: BanyanCid) -> Result<Vec<u8>, DataStoreError> {
        let rip_cid = RipCid::from(cid.clone());
        self.get_block(&rip_cid).await.map_err(|e| e.into())
    }

    async fn store(
        &mut self,
        cid: BanyanCid,
        data: Vec<u8>,
        immediate: bool,
    ) -> Result<(), DataStoreError> {
        // Save the data to a file
        println!("Storing data to IPFS - value length: {}", data.len());
        let rip_cid = RipCid::from(cid.clone());
        let reader = std::io::Cursor::new(data);

        let codec = rip_cid.codec();
        let ipld_codec = IpldCodec::try_from(codec).unwrap();
        let mh_code_int = rip_cid.hash().code();
        let mh_code = match mh_code_int {
            0x16 => MhCode::Sha3_256,
            0x1e => MhCode::Blake3_256,
            _ => MhCode::Blake3_256,
        };
        let res_cid = self
            .put_block(ipld_codec, mh_code, reader)
            .await
            .map_err(|e| e.into())?;
        if res_cid == rip_cid {
            Ok(())
        } else {
            Err(DataStoreError::StoreFailure)
        }
    }
}

impl IpfsRpcClient {
    pub fn new(url: &Url) -> Result<Self, IpfsRpcClientError> {
        let scheme = Scheme::try_from(url.scheme())?;
        let username = url.username();
        let maybe_password = url.password();
        let host_str = url
            .host_str()
            .ok_or(IpfsRpcClientError::Url(url::ParseError::EmptyHost))?;
        let port = url.port().unwrap_or(5001);
        let client = match maybe_password {
            Some(password) => IpfsClient::from_host_and_port(scheme, host_str, port)?
                .with_credentials(username, password),
            None => IpfsClient::from_host_and_port(scheme, host_str, port)?,
        };
        Ok(Self(client))
    }
    /// Add raw data to Ipfs. This will implement chunking for you
    /// Do not use over data where you need control over codecs and chunking
    /// # Arguments
    /// * code: the multihash code to use for the block
    /// * data: the data to add. This can be anything that implements Read. Should be safely passable between threads
    /// # Returns
    /// * the Cid of the data
    // NOTE: this does not support ALL MhCodes. If an unsupported code is passed, it will use our
    // default of blake3
    pub async fn add_data<R>(&self, code: MhCode, data: R) -> Result<RipCid, IpfsRpcClientError>
    where
        R: Read + Send + Sync + 'static + Unpin,
    {
        let hash = match code {
            MhCode::Blake3_256 => "blake3",
            MhCode::Sha3_256 => "sha3-256",
            _ => DEFAULT_MH_TYPE,
        };

        let mut options = AddRequest::default();
        options.hash = Some(hash);
        options.cid_version = Some(DEFAULT_CID_VERSION);

        let response = self.add_with_options(data, options).await?;
        let cid = RipCid::from_str(&response.hash)?;

        Ok(cid)
    }

    /// Put a block to the RPC endpoint. Provides control over the codec and multihash
    /// # Arguments
    /// * codec: the codec to use for the block
    /// * code: the multihash code to use for the block
    /// * data: the data to add. This can be anything that implements Read. Should be safely passable between threads
    /// # Returns
    /// * the Cid of the data
    /// Note: this does not support ALL MhCodes. If an unsupported code is passed, it will use our
    /// default of blake3
    pub async fn put_block<R>(
        &self,
        codec: IpldCodec,
        code: MhCode,
        data: R,
    ) -> Result<RipCid, IpfsRpcClientError>
    where
        R: Read + Send + Sync + 'static + Unpin,
    {
        // TODO: janky, i would like a more robust codec impl that aligns with the ipfs rpc api
        let cic_codec = match codec {
            IpldCodec::DagCbor => "dag-cbor",
            IpldCodec::DagJson => "dag-json",
            IpldCodec::DagPb => "dag-pb",
            IpldCodec::Raw => "raw",
        };

        // TODO: again, there's not just an easy way to match this to whatver the ipfs api expects
        //  For now let's just support blake3 and sha3-256
        let mhtype = match code {
            MhCode::Blake3_256 => "blake3",
            MhCode::Sha3_256 => "sha3-256",
            _ => DEFAULT_MH_TYPE,
        };

        let mut options = BlockPutRequest::default();
        options.mhtype = Some(mhtype);
        options.cid_codec = Some(cic_codec);
        options.pin = Some(true);

        let response = self.block_put_with_options(data, options).await?;

        let hash = response.key;

        let cid = RipCid::from_str(&hash)?;

        Ok(cid)
    }

    /// Remove a block from the IPFS node if it is pinned
    pub async fn rm_block(&self, cid: &RipCid, recursive: bool) -> Result<(), IpfsRpcClientError> {
        let rms = self
            .pin_rm(&format!("{}", cid.to_string()), recursive)
            .await?
            .pins;
        // Check if the cid is removed
        if rms.contains(&cid.to_string()) {
            Ok(())
        } else {
            Err(IpfsRpcClientError::Default(anyhow::anyhow!(
                "Failed to remove cid: {}",
                cid.to_string()
            )))
        }
    }

    /// Check if the RPC endpoint is pinning the specified CID
    pub async fn has_block(&self, cid: &RipCid) -> Result<bool, IpfsRpcClientError> {
        let response = self
            .pin_ls(Some(&format!("{}", cid.to_string())), None)
            .await?;
        let keys = response.keys;
        // Check if the cid is pinned
        Ok(keys.contains_key(&cid.to_string()))
    }

    /// Get Block from IPFS
    pub async fn get_block(&self, cid: &RipCid) -> Result<Vec<u8>, IpfsRpcClientError> {
        let stream = self.block_get(&cid.to_string());

        let block_data = stream.map_ok(|chunk| chunk.to_vec()).try_concat().await?;
        Ok(block_data)
    }

    pub async fn get_block_send_safe(&self, cid: &RipCid) -> Result<Vec<u8>, IpfsRpcClientError> {
        let cid = cid.clone();
        let client = self.clone();
        let response = tokio::task::spawn_blocking(move || {
            tokio::runtime::Handle::current()
                .block_on(client.get_block(&cid))
                .map_err(|e| IpfsRpcClientError::from(e))
        })
        .await
        .map_err(|e| {
            IpfsRpcClientError::Default(
                anyhow::anyhow!("blockstore tokio runtime error: {e}").into(),
            )
        })??;

        Ok(response)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum IpfsRpcClientError {
    #[error("default error: {0}")]
    Default(#[from] anyhow::Error),
    #[error("url parse error")]
    Url(#[from] url::ParseError),
    #[error("http error")]
    Http(#[from] http::Error),
    #[error("Failed to parse scheme")]
    Scheme(#[from] http::uri::InvalidUri),
    #[error("Failed to build client: {0}")]
    Client(#[from] ipfs_api_backend_hyper::Error),
    #[error("cid error")]
    Cid(#[from] wnfs::common::libipld::cid::Error),
}

// For now, just catch all errors and call than as StoreFailures
impl Into<banyanfs::stores::DataStoreError> for IpfsRpcClientError {
    fn into(self) -> banyanfs::stores::DataStoreError {
        match self {
            IpfsRpcClientError::Default(_) => DataStoreError::StoreFailure,
            IpfsRpcClientError::Url(_) => DataStoreError::StoreFailure,
            IpfsRpcClientError::Http(_) => DataStoreError::StoreFailure,
            IpfsRpcClientError::Scheme(_) => DataStoreError::StoreFailure,
            IpfsRpcClientError::Client(_) => DataStoreError::StoreFailure,
            IpfsRpcClientError::Cid(_) => DataStoreError::StoreFailure,
        }
    }
}

mod tests {
    use super::*;

    /// Generate a random 1 KB reader
    fn random_reader() -> impl Read {
        use rand::Rng;
        use std::io::Cursor;
        let mut rng = rand::thread_rng();
        let data: Vec<u8> = (0..1024).map(|_| rng.gen()).collect();
        Cursor::new(data)
    }

    #[tokio::test]
    async fn test_add_data_sha3_256() {
        let ipfs = IpfsRpcClient::default();
        let data = random_reader();
        let mh_code = MhCode::Sha3_256;
        let cid = ipfs.add_data(mh_code, data).await.unwrap();
        assert_eq!(cid.version(), libipld::cid::Version::V1);
        assert_eq!(IpldCodec::try_from(cid.codec()).unwrap(), IpldCodec::Raw);
        assert_eq!(cid.hash().code(), 0x16);
    }

    #[tokio::test]
    async fn test_add_data_blake3_256() {
        let ipfs = IpfsRpcClient::default();
        let data = random_reader();
        let mh_code = MhCode::Blake3_256;
        let cid = ipfs.add_data(mh_code, data).await.unwrap();
        assert_eq!(cid.version(), libipld::cid::Version::V1);
        assert_eq!(IpldCodec::try_from(cid.codec()).unwrap(), IpldCodec::Raw);
        assert_eq!(cid.hash().code(), 0x1e);
    }

    #[tokio::test]
    async fn test_put_block_sha3_256_raw() {
        let ipfs = IpfsRpcClient::default();
        let data = random_reader();
        let mh_code = MhCode::Sha3_256;
        let codec = IpldCodec::Raw;
        let cid = ipfs.put_block(codec, mh_code, data).await.unwrap();
        assert_eq!(cid.version(), libipld::cid::Version::V1);
        assert_eq!(IpldCodec::try_from(cid.codec()).unwrap(), IpldCodec::Raw);
        assert_eq!(cid.hash().code(), 0x16);
    }

    #[tokio::test]
    async fn test_put_block_blake3_256_raw() {
        let ipfs = IpfsRpcClient::default();
        let data = random_reader();
        let mh_code = MhCode::Blake3_256;
        let codec = IpldCodec::Raw;
        let cid = ipfs.put_block(codec, mh_code, data).await.unwrap();
        assert_eq!(cid.version(), libipld::cid::Version::V1);
        assert_eq!(IpldCodec::try_from(cid.codec()).unwrap(), IpldCodec::Raw);
        assert_eq!(cid.hash().code(), 0x1e);
    }
    #[tokio::test]
    async fn test_put_block_sha3_256_dag_cbor() {
        let ipfs = IpfsRpcClient::default();
        let data = random_reader();
        let mh_code = MhCode::Sha3_256;
        let codec = IpldCodec::DagCbor;
        let cid = ipfs.put_block(codec, mh_code, data).await.unwrap();
        assert_eq!(cid.version(), libipld::cid::Version::V1);
        assert_eq!(
            IpldCodec::try_from(cid.codec()).unwrap(),
            IpldCodec::DagCbor
        );
        assert_eq!(cid.hash().code(), 0x16);
    }

    #[tokio::test]
    async fn test_put_block_blake3_256_dag_cbor() {
        let ipfs = IpfsRpcClient::default();
        let data = random_reader();
        let mh_code = MhCode::Blake3_256;
        let codec = IpldCodec::DagCbor;
        let cid = ipfs.put_block(codec, mh_code, data).await.unwrap();
        assert_eq!(cid.version(), libipld::cid::Version::V1);
        assert_eq!(
            IpldCodec::try_from(cid.codec()).unwrap(),
            IpldCodec::DagCbor
        );
        assert_eq!(cid.hash().code(), 0x1e);
    }
}
