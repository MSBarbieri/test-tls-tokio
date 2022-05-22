extern crate tokio;

use rustls_pemfile::{certs, rsa_private_keys};
use std::path::Path;
use std::{
    fs::File,
    io::{self, BufReader},
    sync::Arc,
};

use anyhow::Result;
use tokio::{
    io::{split, AsyncReadExt, AsyncWriteExt},
    net::TcpListener,
    sync::mpsc::channel,
};
use tokio_rustls::{
    rustls::{Certificate, PrivateKey, ServerConfig},
    TlsAcceptor,
};

fn load_certs(path: &Path) -> io::Result<Vec<Certificate>> {
    certs(&mut BufReader::new(File::open(path)?))
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "invalid cert"))
        .map(|mut certs| certs.drain(..).map(Certificate).collect())
}

fn load_keys(path: &Path) -> io::Result<Vec<PrivateKey>> {
    rsa_private_keys(&mut BufReader::new(File::open(path)?))
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "invalid key"))
        .map(|mut keys| keys.drain(..).map(PrivateKey).collect())
}

#[tokio::main]
async fn main() -> Result<()> {
    let listener = TcpListener::bind("0.0.0.0:8080").await?;

    let certs = load_certs(Path::new(""))?;
    let mut keys = load_keys(Path::new(""))?;
    let config = ServerConfig::builder()
        .with_safe_defaults()
        .with_no_client_auth()
        .with_single_cert(certs, keys.remove(0))
        .map_err(|err| io::Error::new(io::ErrorKind::InvalidInput, err))?;
    let acceptor = TlsAcceptor::from(Arc::new(config));
    let (tx, mut rx) = channel(32);
    loop {
        let (socket, _) = listener.accept().await?;
        let _acceptor = acceptor.clone();

        let fut = async move {
            let stream = _acceptor.accept(socket).await?;
            let (mut reader, mut writer) = split(stream);
            writer.flush().await?;
            let mut buf = [0; 1024];
            match reader.read(&mut buf).await {
                Ok(n) if n == 0 => eprintln!("empty socket data"),
                Ok(n) => {
                    let foo = &buf[0..n];
                    println!("{:?}", String::from_utf8(foo.into()));
                }
                Err(e) => {
                    eprintln!("socket failed; err = {:?}", e);
                }
            }
            Ok(()) as io::Result<()>
        };

        let sender = tx.clone();
        tokio::spawn(async move {
            if let Err(err) = fut.await {
                eprintln!("{:?}", err);
                sender.send(true).await;
            }
            sender.send(false).await;
        });
        if let Some(message) = rx.recv().await {
            if message == true {
                break;
            }
        }
    }

    return Ok(());
}
