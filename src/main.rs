use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use libp2p::{
    identity::Keypair,
    swarm::{Swarm, SwarmEvent},
    PeerId,
    Transport,
    NetworkBehaviour, 
    tokio::codec::{Framed, LinesCodec},
};
use crdt::{CRDT, Operation};
use tokio::sync::mpsc;

// Define document data structure
struct Document {
    content: String,
    crdt: CRDT<String>,
    cursor_positions: HashMap<PeerId, usize>,
}

// Define message types
enum Message {
    Change(Operation),
    CursorPosition(usize),
    Join,
    Leave,
}

// Implement NetworkBehaviour trait for message exchange
struct TextEditorBehaviour {
    document: Arc<Mutex<Document>>,
    tx: mpsc::Sender<(PeerId, Vec<u8>)>,
}

impl NetworkBehaviour for TextEditorBehaviour {
    type ProtocolsHandler = <LinesCodec as libp2p::tokio::codec::Encoder>::LinesHandler;
    type OutEvent = Message;
    type InEvent = Message;

    async fn poll<T>(
        &mut self,
        _: &mut Swarm<LinesCodec>,
        mut cx: &mut context::Context,
    ) -> Poll<Extract<Self::OutEvent, T>> {
        if let Ok(message) = self.tx.recv().await {
            Poll::Ready(Extract(message))
        } else {
            Poll::Pending
        }
    }

    async fn handle_in_event(&mut self, peer_id: PeerId, message: Message) {
        match message {
            Message::Change(operation) => {
                self.document.lock().unwrap().crdt.apply_operation(&operation);
                // Update document content and user interface
            }
            Message::CursorPosition(position) => {
                self.document.lock().unwrap().cursor_positions.insert(peer_id, position);
                // Update cursor positions on the user interface
            }
            Message::Join => {
                // Send document content and current cursor positions to the new peer
            }
            Message::Leave => {
                self.document.lock().unwrap().cursor_positions.remove(&peer_id);
                // Update user interface to remove leaving peer's cursor
            }
        }
    }
}

// Main function
#[tokio::main]
async fn main() {
    // Generate keypair and create local PeerId
    let keypair = Keypair::generate();
    let local_peer_id = PeerId::from_public_key(&keypair.public());

    // Create a new Swarm for peer-to-peer communication
    let transport = libp2p::build_tcp_transport().unwrap();
    let swarm = Swarm::new(
        transport,
        local_peer_id,
        TextEditorBehaviour {
            document: Arc::new(Mutex::new(Document {
                content: String::new(),
                crdt: CRDT::new(),
                cursor_positions: HashMap::new(),
            })),
            tx: tx.clone(),
        },
    );

    // Define channels for sending and receiving messages
    let (tx, rx) = mpsc::channel::<(PeerId, Vec<u8>)>(100);

    // Start the Swarm and listen for incoming connections
    swarm.listen_on("/ip4/0.0.0.0/tcp/0").await.unwrap();

    // Broadcast initial document content and cursor positions
    send_join_message(&swarm, &tx, Some(local_peer_id)).await;

    loop {
        // Receive incoming peer connections
        match swarm.next().await {
            SwarmEvent::NewListenAddr { address, .. } => {
                println!("Listening on: {}", address);
            }
            SwarmEvent::ConnectionEstablished { peer_id, .. } => {
                println!("Connected to peer: {}", peer_id);
                send_join_message(&swarm, &tx, Some(peer_id)).await;
            }
            SwarmEvent::ConnectionClosed { peer_id, .. } => {
                println!("Connection closed: {}", peer_id);
            }
            _ => {}
        }

        // Handle user input and send change messages
        // ...
    }
}

async fn send_join_message(swarm: &Swarm<LinesCodec>, tx: &mpsc::Sender<(PeerId, Vec<u8>)>, peer_id: Option<PeerId>) {
    let document = swarm.behaviour().document.lock().unwrap();
    let message = Message::Join;
    let serialized_message = bincode::serialize(&message).unwrap();

    if let Some(specific_peer) = peer_id {
        tx.send((specific_peer, serialized_message.clone())).await.unwrap();
    } else {
        for peer in swarm.connected_peers() {
            tx.send((peer, serialized_message.clone())).await.unwrap();
        }
    }
}

