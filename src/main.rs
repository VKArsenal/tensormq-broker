mod protocol;
mod connection;
mod broker;

use std::sync::Arc;
use broker::Broker;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    println!("Starting TensorMQ ...");

    let broker = Arc::new(Broker::new(1024));
    let addr = "0.0.0.0:59321";

    connection::start_server(addr, broker).await?;

    Ok(())
}