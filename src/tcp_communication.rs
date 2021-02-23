use std::{error::Error, str::from_utf8, sync::Arc, time::Duration};

use keri::{database::lmdb::LmdbEventDatabase, keri::Keri, signer::CryptoBox};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    sync::Mutex,
    time::timeout,
};

pub async fn send(
    message: &[u8],
    address: &str,
    keri_instance: &Keri<LmdbEventDatabase, CryptoBox>,
) -> Result<Vec<u8>, Box<dyn Error>> {
    let mut stream = TcpStream::connect(address.clone()).await?;
    stream.write(message).await?;
    println!("Sent:\n{}\n", from_utf8(message).unwrap());
    let mut buf = [0; 1024];
    stream.readable().await?;
    let n = stream.try_read(&mut buf)?;
    println!("Got:\n{}\n", from_utf8(&buf[..n]).unwrap());

    let res = keri_instance.respond(&buf[..n])?;

    if res.len() != 0 {
        stream.write(&res).await?;
        println!("Sent: {}", String::from_utf8(res.clone()).unwrap());

        let n = match timeout(Duration::from_millis(200), stream.read(&mut buf)).await {
            Ok(n) => n?,
            Err(_) => 0,
        };

        println!("Got:\n{}\n", from_utf8(&buf[..n]).unwrap());
    }

    Ok(buf[..n].to_vec())
}

pub async fn run(
    address: &str,
    keri_instance: Keri<LmdbEventDatabase, CryptoBox>,
) -> Result<(), Box<dyn Error>> {
    let keri_instance = Arc::new(Mutex::new(keri_instance));

    let listener = TcpListener::bind(&address.to_string()).await?;
    println!("Listening on: {}", address);

    loop {
        let (mut socket, _) = listener.accept().await?;
        let keri = Arc::clone(&keri_instance);
        tokio::spawn(async move {
            let mut buf = [0; 1024];

            loop {
                let n = socket
                    .read(&mut buf)
                    .await
                    .expect("failed to read data from socket");

                if n != 0 {
                    let msg = &buf[..n];
                    println!("Got: \n {}\n", String::from_utf8(msg.to_vec()).unwrap());
                    let receipt = keri
                        .lock()
                        .await
                        .respond(msg)
                        .expect("failed while event processing");

                    socket
                        .write_all(&receipt)
                        .await
                        .expect("failed to write data to socket");
                    println!(
                        "Respond with {}\n",
                        String::from_utf8(receipt.clone()).unwrap()
                    );
                }
            }
        });
    }
}
