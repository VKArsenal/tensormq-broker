# TensorMQ Broker

A high-performance, zero-copy TCP message broker specifically engineered for routing multi-gigabyte machine learning tensors, model weights, and high-throughput data streams.

Written completely in asynchronous Rust using Tokio, TensorMQ acts as a transparent, payload-agnostic routing layer. It allows distributed training nodes and inference servers to publish and subscribe to data streams with virtually zero memory overhead.

---

## Core Architecture

* **Zero-Copy Routing:** Payloads are read off the network into `bytes::Bytes` structures. Routing a 10GB tensor to 8 different subscribers simply increments an atomic reference counter (`Arc`), entirely avoiding memory duplication.
* **Payload Agnostic:** The broker only reads the 32-byte TTP v2 header to determine routing boundaries. It does not deserialize the payload, allowing clients to send LZ4-compressed data or raw NumPy arrays transparently.
* **Concurrent Pub/Sub:** Uses `DashMap` for lock-free topic matching and Tokio `broadcast` channels for robust, backpressure-aware 1-to-N fan-out.
* **Advanced Routing:** Supports exact topic matching (`models/resnet/grads`) and wildcard segment matching (`models/*/grads`).
* **Zombie Connection Pruning:** Built-in TCP keepalive timeouts gracefully prune dead connections without leaking file descriptors.

---

## Getting Started

### Prerequisites
* Rust 1.70+ (`cargo`)

### Building from Source

Clone the repository and build the release binary:

```bash
git clone [https://github.com/VKArsenal/tensormq-broker.git](https://github.com/VKArsenal/tensormq-broker.git)
cd tensormq-broker
cargo build --release
```

### Running the Broker
    
Start the broker on the default port (59321):
```Bash
./target/release/tensormq
```

You should see the startup confirmation:

```Plaintext
TensorMQ Broker listening on 0.0.0.0:59321
```

## Protocol Specification (TTP v2)

TensorMQ uses a fixed 32-byte Big-Endian header for network framing, followed by variable-length data segments.

| Bytes   | Field      | Type | Description |
|----------|-----------|------|-------------|
| 0-3      | magic     | u32  | Protocol identifier (0x544D5150 / "TMQP") |
| 4-5      | version   | u16  | Protocol version (currently 2) |
| 6        | msg_type  | u8   | 2: Subscribe, 3: Publish, 4: Meta, 5: Chunk, 7: Heartbeat |
| 7        | flags     | u8   | Bitwise flags (e.g., EOF, LZ4 Compression) |
| 8-15     | stream_id | u64  | Unique ID for reconstructing chunked streams |
| 16-19    | topic_len | u32  | Byte length of the topic string |
| 20-23    | meta_len  | u32  | Byte length of the JSON metadata |
| 24-31    | data_len  | u64  | Byte length of the raw tensor payload |

The broker relies on these exact byte boundaries to slice network streams without deserialization.

## Clients

To interact with the broker, use one of the official language clients:


To be released soon.