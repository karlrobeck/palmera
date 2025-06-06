use futures::stream::StreamExt;
use minio::s3::{
    Client,
    builders::ObjectContent,
    types::{S3Api, ToStream},
};

use crate::traits::{FileStorageError, FileStorageHandler};

pub struct S3Storage {
    client: Client,
}

impl S3Storage {
    pub fn new(client: &Client) -> Self {
        S3Storage {
            client: client.clone(),
        }
    }
}

impl FileStorageHandler for S3Storage {
    async fn upload(&self, id: &str, name: &str, bytes: &[u8]) -> crate::traits::FileResult<()> {
        // create bucket
        let bucket = self
            .client
            .create_bucket(id)
            .send()
            .await
            .map_err(|err| FileStorageError::S3(err))?;

        let content = ObjectContent::from(bytes.to_owned());

        _ = self
            .client
            .put_object_content(bucket.bucket, name, content)
            .send()
            .await
            .map_err(|err| FileStorageError::S3(err))?;

        Ok(())
    }

    async fn download(&self, id: &str, name: &str) -> crate::traits::FileResult<Vec<u8>> {
        let object = self
            .client
            .get_object(id, name)
            .send()
            .await
            .map_err(|err| FileStorageError::S3(err))?;

        let bytes = object
            .content
            .to_segmented_bytes()
            .await
            .map_err(|err| FileStorageError::Io(err))?
            .into_iter()
            .flat_map(|b| b.into_iter())
            .collect::<Vec<u8>>();

        return Ok(bytes);
    }

    async fn list(&self, id: &str) -> crate::traits::FileResult<Vec<String>> {
        let mut stream = self.client.list_objects(id).to_stream().await;

        let mut result: Vec<String> = Vec::new();

        while let Some(entry) = stream.next().await {
            if let Ok(object) = entry {
                for item in object.contents {
                    result.push(item.name.clone());
                }
            }
        }

        Ok(result)
    }
}
