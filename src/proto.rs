use crate::proto::etcdv3::kv_client::KvClient;
use cfg_if::cfg_if;
use eyre::{eyre, Result};
/*
Impl details:
scaffold structure of traefik config using serde.

loop:
On etcd connect crawl structure to build config.
Move to App data after connect.
After edit, diff and push changes?


 */

// this the best way to do this?
cfg_if! {
    if #[cfg(target_arch = "wasm32")] {
        type GrpcChannel = tonic_web_wasm_client::Client;
    }
    else if #[cfg(not(target_arch = "wasm32"))] {
        type GrpcChannel = tonic::transport::Channel;
    }
    else {
        panic!("Unexpected panic, bad build target. somehow.")
    }
}

pub mod etcdv3 {
    tonic::include_proto!("etcdv3");
}
pub mod auth {
    tonic::include_proto!("auth");
}

// pub mod membership {
//     tonic::include_proto!("membership");
// }

pub mod kv {
    tonic::include_proto!("kv");
}

pub struct EtcdClient {
    /// Built on `Self.connect`
    client: Option<KvClient<GrpcChannel>>,
    /// Define on init
    addr: String,
    /// Possibly defined on init - Not supported
    auth: Option<etcdv3::AuthenticateRequest>,
}

impl EtcdClient {
    pub fn new(addr: String) -> Self {
        Self {
            client: None,
            addr,
            auth: None,
        }
    }
    fn new_with_auth(addr: String, auth: etcdv3::AuthenticateRequest) -> Self {
        Self {
            client: None,
            addr,
            auth: Some(auth),
        }
    }

    fn _connected(&self) -> Result<()> {
        match &self.client {
            Some(_) => Ok(()),
            None => Err(eyre!("Client not connected")),
        }
    }

    /// Connects `KVClient`
    pub async fn connect(&mut self) -> Result<()> {
        cfg_if! {
            if #[cfg(target_arch = "wasm32")] {
                let channel = tonic_web_wasm_client::Client::new(format!("http://{}", self.addr)); // TODO: add secure support
                let client = KvClient::new(channel); // This might not init the connection, calling connect is async whereas this is sync
                self.client = Some(client);
            }
            else if #[cfg(not(target_arch = "wasm32"))] {
                let client = KvClient::connect(self.addr.clone()).await?;
                self.client = Some(client);
            }
            else {
                panic!("Unexpected panic, cannot build channel")
            }
        }
        Ok(())
    }

    pub async fn put(&mut self, key: String, value: String) -> Result<()> {
        self._connected()?;

        let req = tonic::Request::new(etcdv3::PutRequest {
            key: key.into_bytes(),
            value: value.into_bytes(),
            lease: 0,
            prev_kv: false,
            ignore_value: false,
            ignore_lease: false,
        });
        self.client.as_mut().unwrap().put(req).await?.into_inner();
        Ok(())
    }

    pub async fn get(&mut self, key: String) -> Result<Option<String>> {
        self._connected()?;

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

        let v = self.client.as_mut().unwrap().range(req).await?.into_inner();
        Ok(None)
    }
    pub async fn get_all(&mut self) -> Result<()> {
        self._connected()?;

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
