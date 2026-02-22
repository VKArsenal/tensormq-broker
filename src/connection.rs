use bytes::BytesMut;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use std::io::{Error, ErrorKind};
use std::sync::Arc;

use crate::protocol::{Header, HEADER_SIZE, MsgType};
use crate::broker::{Broker, TensorMessage};

/// Bind TCP listener and accept incoming connections.
pub async fn start_server(addr: &str, broker: Arc<Broker>) -> std::io::Result<()> {
    let listener = TcpListener::bind(addr).await?;
    println!("TensorMQ Broker listening on: {}", addr);

    loop {
        let (socket, remote_addr) = listener.accept().await?;
        println!("Accepted connection from: {}", remote_addr);

        let broker_clone = broker.clone();

        tokio::spawn(async move {
            if let Err(e) = handle_connection(socket, broker_clone).await {
                eprintln!("Connection error from {}: {}", remote_addr, e);
            }
        });
    }
}

/// Handle each individual client
async fn handle_connection(mut socket: TcpStream, broker: Arc<Broker>) -> std::io::Result<()> {
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

            let topic_str = String::from_utf8_lossy(&topic_bytes).to_string();

            if header.msg_type == MsgType::Publish {
                let msg = TensorMessage {
                    topic: topic_str,
                    meta: meta_bytes,
                    tensor: tensor_bytes,
                };
                broker.publish(msg);
            } else if header.msg_type == MsgType::Subscribe {
                println!("Client subscribed to: {}", topic_str);

                let mut rx = broker.subscribe(&topic_str);

                loop {
                    match rx.recv().await {
                        Ok(msg) => {
                            let mut out_header = BytesMut::with_capacity(HEADER_SIZE);
                            let h = Header {
                                version: 2,
                                msg_type: MsgType::Publish,
                                flags: 0,
                                stream_id: 0, // TODO: Make this dynamic
                                topic_len: msg.topic.len() as u32,
                                meta_len: msg.meta.len() as u32,
                                data_len: msg.tensor.len() as u64,
                            };
                            h.encode(&mut out_header);

                            socket.write_all(&out_header).await?;
                            socket.write_all(msg.topic.as_bytes()).await?;
                            socket.write_all(&msg.meta).await?;
                            socket.write_all(&msg.tensor).await?;
                        }
                        Err(tokio::sync::broadcast::error::RecvError::Lagged(n)) => {
                            eprintln!("Subscriber lagged and missed {} messages", n);
                        }
                        Err(tokio::sync::broadcast::error::RecvError::Closed) => {
                            break;
                        }
                    }
                }
            }
        }
    }
}