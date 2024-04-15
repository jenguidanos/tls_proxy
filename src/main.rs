mod client;
mod config;
mod server;

use crate::config::Config;
use env_logger::Env;
use log::{error, info};
use std::sync::Arc;
use tokio::io;
use tokio::sync::mpsc;
use tokio::sync::Mutex;

// Define type aliases to simplify the complex types
type ChannelSender = mpsc::Sender<Vec<u8>>;
type ChannelReceiver = Arc<Mutex<mpsc::Receiver<Vec<u8>>>>;

/// The entry point for the asynchronous main function.
#[tokio::main]
async fn main() -> io::Result<()> {
    // Initialize the logger for the application.
    setup_logger();

    // Create communication channels for server-client data exchange.
    let (tx_to_server, rx_from_client, tx_to_client, rx_from_server) = create_channels();

    // Launch the server and client in separate asynchronous tasks.
    let conf = config::Config::new().expect("Failed to load config");
    let server_handle = start_server(tx_to_client, rx_from_server, conf.clone());
    let client_handle = start_client(tx_to_server, rx_from_client, conf.clone());

    // Wait for both the client and server to finish executing.
    let _ = tokio::try_join!(server_handle, client_handle);

    Ok(())
}

/// Sets up the logging environment from the environment variables.
fn setup_logger() {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
}

/// Creates communication channels and wraps receivers in `Arc<Mutex>` for safe multi-threaded access.
fn create_channels() -> (
    ChannelSender,
    ChannelReceiver,
    ChannelSender,
    ChannelReceiver,
) {
    // Setup message channels between server and client.
    let (tx_to_server, rx_from_client) = mpsc::channel::<Vec<u8>>(100);
    let (tx_to_client, rx_from_server) = mpsc::channel::<Vec<u8>>(100);

    // Use Arc and Mutex to safely share and mutate the receiver state across threads.
    let rx_from_client = Arc::new(Mutex::new(rx_from_client));
    let rx_from_server = Arc::new(Mutex::new(rx_from_server));

    (tx_to_server, rx_from_client, tx_to_client, rx_from_server)
}

/// Starts the server in a new asynchronous task and handles its result.
fn start_server(
    tx_to_client: ChannelSender,
    rx_from_server: ChannelReceiver,
    conf: Arc<Config>,
) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        // Handle server operation results and log accordingly.
        match server::run_server(tx_to_client, rx_from_server, conf).await {
            Ok(_) => info!("Server terminated successfully."),
            Err(e) => error!("Server terminated with an error: {}", e),
        }
    })
}

/// Starts the client in a new asynchronous task and handles its result.
fn start_client(
    tx_to_server: ChannelSender,
    rx_from_client: ChannelReceiver,
    conf: Arc<Config>,
) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        // Handle client operation results and log accordingly.
        match client::run_client(tx_to_server, rx_from_client, conf).await {
            Ok(_) => info!("Client terminated successfully."),
            Err(e) => error!("Client terminated with an error: {}", e),
        }
    })
}
