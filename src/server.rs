use crate::config::Config;
use log::{debug, error, info};
use std::sync::Arc;
use tokio::io::{self, AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::sync::Mutex;
use tokio::time::{sleep, Duration};
use tokio_rustls::TlsAcceptor;

mod certs; // Import the certs module

pub async fn run_server(
    tx: Sender<Vec<u8>>,
    rx: Arc<Mutex<Receiver<Vec<u8>>>>,
    config: Arc<Config>,
) -> io::Result<()> {
    let (listener, acceptor) = start_tls_server(config).await?;
    loop {
        match listener.accept().await {
            Ok((socket, addr)) => {
                info!("Connection accepted from {}", addr);
                let tx = tx.clone();
                let rx = rx.clone();
                let acceptor = acceptor.clone();
                tokio::spawn(async move {
                    if let Err(e) = handle_tls_connection(socket, acceptor, tx, rx).await {
                        error!("Failed to handle connection: {}", e);
                    }
                });
            }
            Err(e) => {
                error!("Failed to accept connection: {}", e);
                continue;
            }
        }
    }
}

/// Attempts to start the TLS server on a specified port and logs relevant
/// information about its state.
///
/// # Returns
/// Returns a `Result` containing a tuple of `TcpListener` and `TlsAcceptor`,
/// or an `io::Error` if the server cannot start.
async fn start_tls_server(conf: Arc<Config>) -> io::Result<(TcpListener, TlsAcceptor)> {
    let address = format!("{}:{}", conf.tls_proxy_host, conf.tls_proxy_port);
    let mut failure_logged = false;

    info!("Starting TLS Proxy at {}", address);

    loop {
        // Load the TLS configuration
        let tls_conf =
            match certs::load_tls_config("path/to/cert.pem", "path/to/key.pem", "path/to/ca.pem") {
                Ok(tls_conf) => tls_conf,
                Err(e) => {
                    // Changed `e` to `_e` to indicate it's intentionally unused
                    if !failure_logged {
                        error!("TLS Proxy failed to start, retrying... {}", e);
                        failure_logged = true;
                    }
                    sleep(Duration::from_secs(5)).await;
                    continue;
                }
            };

        let acceptor = TlsAcceptor::from(Arc::new(tls_conf));

        match TcpListener::bind(&address).await {
            Ok(listener) => {
                info!("TLS Proxy started");
                return Ok((listener, acceptor));
            }
            Err(e) => {
                if !failure_logged {
                    error!("TLS Proxy failed to start, retrying... {}", e);
                    failure_logged = true;
                }
                sleep(Duration::from_secs(5)).await;
            }
        }
    }
}

/// Handles a single TLS connection.
async fn handle_tls_connection(
    socket: TcpStream,
    acceptor: TlsAcceptor,
    tx: Sender<Vec<u8>>,
    rx: Arc<Mutex<Receiver<Vec<u8>>>>,
) -> io::Result<()> {
    let mut tls_stream = acceptor.accept(socket).await?;
    let mut buf = vec![0; 1024];

    loop {
        let mut rx_ref = rx.lock().await;
        tokio::select! {
            read_result = tls_stream.read(&mut buf) => {
                match read_result {
                    Ok(0) => {
                        info!("Connection closed by peer");
                        break;
                    }
                    Ok(size) => {
                        debug!("Received {} bytes from client", size);
                        if let Err(err) = tx.send(buf[..size].to_vec()).await {
                            error!("Failed to send message: {}", err);
                            break;
                        }
                    }
                    Err(err) => {
                        error!("Error reading from socket: {}", err);
                        break;
                    }
                }
            },
            Some(data) = rx_ref.recv() => {
                if let Err(e) = tls_stream.write_all(&data).await {
                    error!("Failed to write to socket: {}", e);
                    break;
                }
                debug!("Sent data to client");
            }
        }
    }

    Ok(())
}
