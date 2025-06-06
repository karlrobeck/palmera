use minio::s3::{ClientBuilder, creds::StaticProvider, http::BaseUrl};
use palmera_storage::s3::S3Storage;
use palmera_storage::traits::FileStorageHandler;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let base_url = "https://play.min.io".parse::<BaseUrl>()?;

    let static_provider = StaticProvider::new(
        "Q3AM3UQ867SPQQA43P2F",
        "zuf+tfteSlswRu7BJ86wekitnifILbZam1KYY3TG",
        None,
    );

    let client = ClientBuilder::new(base_url.clone())
        .provider(Some(Box::new(static_provider)))
        .build()?;

    let s3_storage = S3Storage::new(&client);

    // Example usage:
    let bucket = "palmera-example-bucket";
    let file_name = "hello.txt";
    let file_content = b"Hello, palmera-storage!";

    // Upload a file
    s3_storage.upload(bucket, file_name, file_content).await?;
    println!("Uploaded {} to bucket {}", file_name, bucket);

    // List files in the bucket
    let files = s3_storage.list(bucket).await?;
    println!("Files in bucket {}: {:?}", bucket, files);

    // Download the file
    let downloaded = s3_storage.download(bucket, file_name).await?;
    println!(
        "Downloaded content: {}",
        String::from_utf8_lossy(&downloaded)
    );

    Ok(())
}
