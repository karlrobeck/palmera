// path format /<id>/<file_name>

#[derive(Debug)]
pub enum FileStorageError {
    Local(std::io::Error),
    S3(minio::s3::error::Error),
}

pub type FileResult<T> = Result<T, FileStorageError>;

/// `FileStorageHandler` is a trait for handling file storage operations.
///
/// It defines methods for uploading, downloading, and listing files within a storage system.
/// Implementations of this trait provide the specific logic for interacting with the underlying storage.
///
/// # Methods
///
/// *   `upload`: Uploads a file to the storage.
/// *   `download`: Downloads a file from the storage.
/// *   `list`: Lists files in the storage.
pub trait FileStorageHandler {
    /// Uploads a file to the storage.
    ///
    /// # Arguments
    ///
    /// *   `id`: The identifier for the file's location or namespace.
    /// *   `name`: The name of the file to be uploaded.
    /// *   `bytes`: The byte content of the file.
    ///
    /// # Returns
    ///
    /// A `FileResult` indicating success or failure.
    fn upload(
        &self,
        id: &str,
        name: &str,
        bytes: &[u8],
    ) -> impl std::future::Future<Output = FileResult<()>> + Send;
    /// Downloads a file from the storage.
    ///
    /// # Arguments
    ///
    /// *   `id`: The identifier for the file's location or namespace.
    /// *   `name`: The name of the file to be downloaded.
    ///
    /// # Returns
    ///
    /// A `FileResult` containing the byte content of the file, or an error if the download fails.
    fn download(
        &self,
        id: &str,
        name: &str,
    ) -> impl std::future::Future<Output = FileResult<Vec<u8>>> + Send;
    /// Lists files in the storage.
    ///
    /// # Arguments
    ///
    /// *   `id`: The identifier for the file's location or namespace.
    ///
    /// # Returns
    ///
    /// A `FileResult` containing a vector of file names, or an error if the listing fails.
    fn list(&self, id: &str) -> impl std::future::Future<Output = FileResult<Vec<String>>> + Send;
}
