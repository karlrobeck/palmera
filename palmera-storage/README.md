# palmera-storage

`palmera-storage` provides file storage backends for the Palmera project. It currently supports local filesystem storage and can be extended to support other backends (e.g., S3).

## Features
- Async file upload, download, and listing
- Pluggable storage handler trait
- Local filesystem implementation

## Usage

Add `palmera-storage` as a dependency in your `Cargo.toml`:

```toml
[dependencies]
palmera-storage = { path = "../palmera-storage" }
```

### Example: Local Storage

```rust
use palmera_storage::{LocalStorage, FileStorageHandler};
use std::path::PathBuf;

#[tokio::main]
async fn main() {
    let storage = LocalStorage::new(PathBuf::from("/tmp/palmera"));
    storage.upload("user1", "file.txt", b"hello world").await.unwrap();
    let data = storage.download("user1", "file.txt").await.unwrap();
    println!("Downloaded: {}", String::from_utf8_lossy(&data));
}
```

## Extending
Implement the `FileStorageHandler` trait for new storage backends.

## License
MIT
