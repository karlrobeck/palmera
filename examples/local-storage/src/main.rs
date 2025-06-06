use palmera_storage::local::LocalStorage;
use palmera_storage::traits::FileStorageHandler;
use tempdir::TempDir;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize local storage with a base directory
    let tmp_dir = TempDir::new("palmera-storage-example")?;
    let storage = LocalStorage::new(tmp_dir.into_path());

    let id = "example"; // directory or namespace
    let name = "test-file.txt";
    let content = b"Hello, Palmera!";

    // Upload the file
    storage.upload(id, name, content).await.unwrap();
    println!("File uploaded successfully: {}/{}", id, name);

    // Download the file
    let downloaded = storage.download(id, name).await.unwrap();
    println!(
        "File downloaded successfully: {}/{} ({})",
        id,
        name,
        String::from_utf8_lossy(&downloaded)
    );

    // List files
    let files = storage.list(id).await.unwrap();
    println!("Files in storage ({}):", id);
    for file in files {
        println!("- {}", file);
    }

    Ok(())
}
