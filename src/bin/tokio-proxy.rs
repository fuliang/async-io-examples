use std::error::Error;

use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncWriteExt, self};
use futures::FutureExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let listen_addr = "127.0.0.1:8001".to_string();
    let server_addr = "127.0.0.1:8000".to_string();

    println!("Listening on: {}", listen_addr);
    println!("Proxy to: {}", server_addr);

    let listener = TcpListener::bind(listen_addr).await?;
    while let Ok((inbound, _)) = listener.accept().await {
        let transfer = transfer(inbound, server_addr.clone()).map(|r| {
            if let Err(e) = r {
                println!("Failed to transfer; error={}", e);
            }
        });
        tokio::spawn(transfer);
    }

    Ok(())
}

async fn transfer(mut inbound: TcpStream, proxy_addr: String) -> Result<(), Box<dyn Error>> {
    let mut outbound = TcpStream::connect(proxy_addr).await?;
    let (mut ri, mut wi) = inbound.split();
    let (mut ro, mut wo) = outbound.split();

    let client_to_server = async {
        io::copy(&mut ri, &mut wo).await?;
        wo.shutdown().await
    };

    let server_to_client = async {
        io::copy(&mut ro, &mut wi).await?;
        wi.shutdown().await
    };
    
    tokio::try_join!(client_to_server, server_to_client)?;

    Ok(())
}