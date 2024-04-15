use log::{info, warn};
use std::sync::Arc;
use tokio::io::{self, AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::sync::Mutex;

use crate::config::Config;

/// Initiates the client process to continuously attempt to connect to the server,
/// handle connections, and manage reconnection attempts upon connection loss.
///
/// # Arguments
///
/// * `tx` - Sender part of a channel to send data received from the server to the processing unit.
/// * `rx` - Receiver wrapped in an Arc and Mutex for receiving data to be sent to the server.
pub async fn run_client(
    tx: Sender<Vec<u8>>,
    rx: Arc<Mutex<Receiver<Vec<u8>>>>,
    conf: Arc<Config>,
) -> io::Result<()> {
    let server_address = format!("{}:{}", conf.ccs_client_ip, conf.ccs_client_port);
    let mut connection_lost_once = false;

    loop {
        if !connection_lost_once {
            info!("Connecting to CCS server at {}", server_address);
        }
        match TcpStream::connect(server_address.clone()).await {
            Ok(stream) => {
                if connection_lost_once {
                    info!("Client reconnect.");
                } else {
                    info!("Client connected.");
                }
                if handle_connection(stream, tx.clone(), rx.clone())
                    .await
                    .is_err()
                {
                    warn!("CCS Server lost, reconnecting...");
                    connection_lost_once = true;
                }
            }
            Err(_) => {
                if !connection_lost_once {
                    warn!("Failed to connect to CCS Server, reconnecting...");
                    connection_lost_once = true;
                }
                tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
            }
        }
    }
}

/// Handles the active connection to the server, reading from and writing to the
/// stream as data is available.
///
/// # Arguments
///
/// * `stream` - The TCP stream associated with the connection to the server.
/// * `tx` - Sender for sending data received from the server.
/// * `rx` - Receiver for getting data to be sent to the server, wrapped in Arc and
///          Mutex for safe concurrent access.
///
/// # Returns
///
/// This function returns an `io::Result<()>` indicating the success or failure
/// of the connection handling.
async fn handle_connection(
    mut stream: TcpStream,
    tx: Sender<Vec<u8>>,
    rx: Arc<Mutex<Receiver<Vec<u8>>>>,
) -> io::Result<()> {
    let mut buf = vec![0; 1024];
    loop {
        let mut rx_ref = rx.lock().await;
        tokio::select! {
            read_result = stream.read(&mut buf) => {
                let n = read_result?;
                if n == 0 { return Err(io::Error::new(io::ErrorKind::BrokenPipe, "Connection closed by peer")); }
                tx.send(buf[..n].to_vec()).await.expect("Failed to send message to TLS server");
            },
            Some(data) = rx_ref.recv() => {
                stream.write_all(&data).await?;
            }
        }
    }
}
