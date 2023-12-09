use libp2p::{
    identity::{self, Keypair},
    PeerId,
    floodsub::{Topic},
};
use log::{error, info};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use tokio::{fs, io::AsyncBufReadExt, sync::mpsc};

const STORAGE_FILE_PATH: &str = "./data.json";

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync + 'static>>;
type FileData = Vec<File>;

static KEYS: Lazy<identity::Keypair> = Lazy::new(|| identity::Keypair::generate_ed25519());
static PEER_ID: Lazy<PeerId> = Lazy::new(|| PeerId::from(KEYS.public()));
static TOPIC: Lazy<Topic> = Lazy::new(|| Topic::new("files"));

#[derive(Serialize, Deserialize, Debug)]
struct File {
    id: usize,
    name: String,
    description: String,
    public: bool
}

#[derive(Serialize, Deserialize, Debug)]
enum ListMode {
    All,
    One(String)
}

#[derive(Serialize, Deserialize, Debug)]
struct ListRequest {
    mode: ListMode
}

#[derive(Serialize, Deserialize, Debug)]
struct ListResponse {
    mode: ListMode,
    data: File,
    receiver: String
}

enum EventType {
    Response(ListResponse),
    Input(String)
}


#[tokio::main]
async fn main() {
    pretty_env_logger::init();

    info!("My Peer id is {:?}", PEER_ID);
    let (response_sender, mut response_rcv) = mpsc::unbounded_channel::<String>();
    let authKeys = Keypair::generate_ed25519();
    println!("Here are your keypairs {:?}", authKeys);


}