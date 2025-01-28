#![allow(dead_code)] // TODO: remove
#![allow(unused)]

use crate::proto::etcdv3::kv_client::KvClient;
use crate::proto::*;
use cfg_if::cfg_if;
use eyre::{eyre, Result};

#[cfg(not(target_arch = "wasm32"))]
use tonic::transport::Channel as GrpcChannel;
#[cfg(target_arch = "wasm32")]
use tonic_web_wasm_client::Client as GrpcChannel;
use url::{ParseError, Url};
/*
Impl details:
scaffold structure of traefik config using serde.

loop:
On etcd connect crawl structure to build config.
Move to App data after connect.
After edit, diff and push changes?


 */

pub struct EtcdClient {
    /// Built on `Self.connect`
    client: KvClient<GrpcChannel>,
    /// Define on init
    addr: String,
    /// Auth packet
    auth: Option<etcdv3::AuthenticateRequest>,
}

impl EtcdClient {
    pub fn new(addr: String) -> Result<Self> {
        Self::_valid_url(addr.clone())?;
        let client = Self::_build_channel(addr.clone())?;
        Ok(Self {
            client,
            addr,
            auth: None,
        })
    }
    pub fn new_with_auth(addr: String, auth: etcdv3::AuthenticateRequest) -> Result<Self> {
        todo!("Auth is coming at a later time.")
    }
    fn _valid_url(url: String) -> Result<()> {
        match Url::parse(&url) {
            Ok(url) =>{
                match url.scheme() { 
                    "http" | "https" => {}
                    _ => return Err(eyre!("Unexpected Scheme. Expected http or https. Got: '{}'", url.scheme())),
                }
                match url.path() { 
                    "/" => {}
                    _ => return Err(eyre!("Got unexpected path")),
                }
                Ok(())
            },
            Err(e) => Err(eyre!("Invalid URL: {}", e)),
        }
    }

    fn _build_channel(addr: String) -> Result<KvClient<GrpcChannel>> {
        cfg_if! {
            if #[cfg(target_arch = "wasm32")] {
                let channel = GrpcChannel::new(addr);
                Ok(KvClient::new(channel))
            }
            else if #[cfg(not(target_arch = "wasm32"))] {
                let channel = GrpcChannel::builder(addr.parse()?).connect_lazy();
                Ok(KvClient::new(channel))
            }
        }
    }
    fn _build_auth_channel(addr: String) -> Result<KvClient<GrpcChannel>> {
        todo!("Auth is coming at a later time.")
    }

    pub async fn put(&mut self, key: String, value: String) -> Result<()> {
        let req = tonic::Request::new(etcdv3::PutRequest {
            key: key.into_bytes(),
            value: value.into_bytes(),
            lease: 0,
            prev_kv: false,
            ignore_value: false,
            ignore_lease: false,
        });
        self.client.put(req).await?;
        Ok(())
    }

    pub async fn get(&mut self, key: String) -> Result<Option<String>> {
        let req = tonic::Request::new(etcdv3::RangeRequest {
            key: key.clone().into_bytes(),
            range_end: key.into_bytes(),
            limit: 0,
            revision: 0,
            sort_order: 0,
            sort_target: 0,
            serializable: false,
            keys_only: false,
            count_only: false,
            min_mod_revision: 0,
            max_mod_revision: 0,
            min_create_revision: 0,
            max_create_revision: 0,
        });

        let _v = self.client.range(req).await?.into_inner();
        Ok(None)
    }
    pub async fn get_all(&mut self) -> Result<()> {
        let key_bytes = "traefik".to_string().into_bytes();
        let end_bytes = get_prefix(key_bytes.clone());

        let req = tonic::Request::new(etcdv3::RangeRequest {
            key: key_bytes,
            range_end: end_bytes,
            limit: 0,
            revision: 0,
            sort_order: 0,
            sort_target: 0,
            serializable: false,
            keys_only: false,
            count_only: false,
            min_mod_revision: 0,
            max_mod_revision: 0,
            min_create_revision: 0,
            max_create_revision: 0,
        });
        Ok(())
    }
}

fn get_prefix(key: Vec<u8>) -> Vec<u8> {
    let mut end = key.to_vec(); // Create a mutable copy of the input key

    for i in (0..end.len()).rev() {
        if end[i] < 0xff {
            end[i] += 1;
            end.truncate(i + 1); // Truncate the vector to remove any trailing bytes
            return end;
        }
    }

    // next prefix does not exist (e.g., 0xffff);
    // default to noPrefixEnd (you need to define noPrefixEnd appropriately)
    const NO_PREFIX_END: &[u8] = b""; // Adjust this as needed
    NO_PREFIX_END.to_vec()
}
