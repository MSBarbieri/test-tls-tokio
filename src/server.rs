extern crate tokio;

use anyhow::Result;
use tokio::{
    io::{split, AsyncReadExt},
    net::TcpListener,
};

#[tokio::main]
async fn main() -> Result<()> {
    let mut listener = TcpListener::bind("0.0.0.0:8080").await?;

    loop {
        let (mut socket, addr) = listener.accept().await?;
        let mut buf = [0; 1024];

        match socket.read(&mut buf).await {
            Ok(n) if n == 0 => eprintln!("empty socket data"),
            Ok(n) => {
                let foo = &buf[0..n];
                println!("{:?}", String::from_utf8(foo.into())?);
            }
            Err(e) => {
                eprintln!("socket failed; err = {:?}", e);
            }
        }
    }
    Ok(())
}
