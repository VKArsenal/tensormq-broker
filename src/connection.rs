use std::io::{Error, ErrorKind};
use bytes::BytesMut;
use tokio::io::AsyncReadExt;
use tokio::net::{TcpListener, TcpStream};
use crate::protocol::{Header, HEADER_SIZE};

/// Bind TCP listener and accept incoming connections.
pub async fn start_server(addr: &str) -> std::io::Result<()> {
    let listener = TcpListener::bind(addr).await?;
    println!("TensorMQ Broker listening on: {}", addr);

    loop {
        let (socker, remote_addr) = listener.accept().await?;
        println!("Accepted connection from: {}", remote_addr);

        tokio::spawn(async move {
            if let Err(e) = handle_connection(socker).await {
                eprintln!("Connection error from {}: {}", remote_addr, e);
            }
        });
    }
}

/// Handle each individual client
async fn handle_connection(mut socket: TcpStream) -> std::io::Result<()> {
    // let mut buffer = BytesMut::with_capacity(8192);
    let mut header_buf = [0u8; HEADER_SIZE];
    loop {
        let bytes_read = socket.read_exact(&mut header_buf).await;
        match bytes_read {
            Ok(_) => {}
            Err(e) if e.kind() == ErrorKind::UnexpectedEof => {
                println!("Client gracefully disconnected");
                return Ok(());
            }
            Err(e) => return Err(e),
        }

        let mut buf_slice = &header_buf[..];
        let header = match Header::decode(&mut buf_slice) {
            Ok(h) => h,
            Err(e) => {
                eprintln!("Protocol Framing Error: {}", e);
                return Err(Error::new(ErrorKind::InvalidData, e.to_string()));
            }
        };

        println!("Header Received: {:?}", header);

        let payload_size = (header.topic_len + header.meta_len) as usize + header.data_len as usize;

        if payload_size > 0 {
            let mut payload_buf = BytesMut::zeroed(payload_size);

            socket.read_exact(&mut payload_buf).await?;

            // Zero Copy Slicing
            let topic_bytes = payload_buf.split_to(header.topic_len as usize).freeze();
            let meta_bytes = payload_buf.split_to(header.meta_len as usize).freeze();
            let tensor_bytes = payload_buf.freeze();

            let topic_str = String::from_utf8_lossy(&topic_bytes);

            println!("Topic: {}", topic_str);
            println!("Meta Length: {} bytes", meta_bytes.len());
            println!("Tensor Size: {} bytes", tensor_bytes.len());

            // TODO: PUB SUB ROUTING
        }
    }
}