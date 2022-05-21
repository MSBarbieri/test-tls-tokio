use tokio::{io::AsyncWriteExt, net::TcpStream};

use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    let mut stream = TcpStream::connect("0.0.0.0:8080").await?;

    stream.write_all(b"hello server").await?;
    Ok(())
}
