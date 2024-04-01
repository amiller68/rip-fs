use std::ops::Deref;
use std::str::FromStr;

pub use banyanfs::codec::Cid as BanyanCid;
pub use libipld::cbor::DagCborCodec;
pub use libipld::cid::multihash::Code as MhCode;
pub use libipld::Cid;
pub use libipld::Ipld;
pub use libipld::IpldCodec;

#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct RipCid(Cid);

impl Deref for RipCid {
    type Target = Cid;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<Cid> for RipCid {
    fn from(cid: Cid) -> Self {
        RipCid(cid)
    }
}

impl Into<Cid> for RipCid {
    fn into(self) -> Cid {
        self.0
    }
}

impl From<BanyanCid> for RipCid {
    fn from(cid: BanyanCid) -> Self {
        let url = cid.as_base64url_multicodec();
        RipCid(Cid::from_str(&url).unwrap())
    }
}

impl Into<BanyanCid> for RipCid {
    fn into(self) -> BanyanCid {
        let bytes = self.0.to_bytes();
        match BanyanCid::parse(&bytes) {
            Ok((_, cid)) => cid,
            Err(_) => panic!("Invalid Cid"),
        }
    }
}

impl FromStr for RipCid {
    type Err = <Cid as FromStr>::Err;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(RipCid(Cid::from_str(s)?))
    }
}
