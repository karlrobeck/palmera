use std::{io, path::PathBuf};

use tokio::io::AsyncWriteExt;

use crate::traits::{FileStorageError, FileStorageHandler};

pub struct LocalStorage {
    base_dir: PathBuf,
}

impl LocalStorage {
    pub fn new(dir: PathBuf) -> Self {
        Self { base_dir: dir }
    }
}

impl FileStorageHandler for LocalStorage {
    async fn upload(&self, id: &str, name: &str, bytes: &[u8]) -> crate::traits::FileResult<()> {
        tokio::fs::create_dir(self.base_dir.join(id))
            .await
            .map_err(|err| FileStorageError::Local(err))?;

        let path = self.base_dir.join(id).join(name);

        let mut file = tokio::fs::File::create_new(path)
            .await
            .map_err(|err| FileStorageError::Local(err))?;

        Ok(file
            .write_all(bytes)
            .await
            .map_err(|err| FileStorageError::Local(err))?)
    }

    async fn download(&self, id: &str, name: &str) -> crate::traits::FileResult<Vec<u8>> {
        let path = self.base_dir.join(id).join(name);

        let file = tokio::fs::read(path)
            .await
            .map_err(|err| FileStorageError::Local(err))?;

        Ok(file)
    }

    async fn list(&self, id: &str) -> crate::traits::FileResult<Vec<String>> {
        let dir = self.base_dir.join(id);

        let mut dir_list = tokio::fs::read_dir(dir)
            .await
            .map_err(|err| FileStorageError::Local(err))?;

        let mut files = vec![];

        while let Some(entry) = dir_list
            .next_entry()
            .await
            .map_err(|err| FileStorageError::Local(err))?
        {
            let file_name = entry.file_name();
            let file_name_str = file_name
                .to_str()
                .ok_or(FileStorageError::Local(io::ErrorKind::NotFound.into()))?;
            files.push(file_name_str.to_string());
        }

        Ok(files)
    }
}
