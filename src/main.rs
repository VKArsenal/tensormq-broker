mod protocol;
mod connection;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    println!("Starting TensorMQ ...");
    let addr = "0.0.0.0:59321";

    connection::start_server(addr).await?;

    Ok(())
}