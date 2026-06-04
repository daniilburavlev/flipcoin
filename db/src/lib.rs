use std::{path::Path, sync::Arc};

use async_rocksdb::{AsyncRocksBuilder, AsyncRocksDB, AsyncRocksError};
use common::{AppResult, error::AppError};
use serde::{Serialize, de::DeserializeOwned};

#[derive(Clone)]
pub struct DB {
    pub db: Arc<AsyncRocksDB>,
}

impl DB {
    pub async fn open(path: &Path, cfs: Vec<&str>) -> AppResult<Self> {
        let mut builder = AsyncRocksBuilder::new();
        for cf in cfs {
            builder = builder.add_column_family(cf);
        }
        let db = builder.open(path).await.map_err(|_| AppError::DB)?;
        Ok(Self { db: Arc::new(db) })
    }

    pub async fn put<S: Serialize>(&self, key: String, value: &S, cf: &str) -> AppResult<()> {
        self.db
            .put(key, to_b(value)?, Some(cf))
            .await
            .map_err(to_err)?;
        Ok(())
    }

    pub async fn get<D: DeserializeOwned>(&self, key: String, cf: &str) -> AppResult<Option<D>> {
        match self.db.get(key, Some(cf), None).await.map_err(to_err)? {
            Some(bytes) => Ok(Some(from_b(bytes)?)),
            None => Ok(None),
        }
    }

    pub async fn prefix_all<D: DeserializeOwned>(
        &self,
        prefix: String,
        cf: &str,
    ) -> AppResult<Vec<D>> {
        let mut values = vec![];
        for (_, v) in self
            .db
            .prefix_all(prefix, Some(cf), None)
            .await
            .map_err(to_err)?
        {
            values.push(from_b(v)?);
        }
        Ok(values)
    }

    pub async fn multi_get<D: DeserializeOwned>(
        &self,
        keys: Vec<String>,
        cf: &str,
    ) -> AppResult<Vec<D>> {
        let mut values = Vec::with_capacity(keys.len());
        for v in self
            .db
            .multi_get(keys, Some(cf), None)
            .await
            .map_err(to_err)?
            .into_iter()
            .flatten()
        {
            values.push(from_b(v)?);
        }
        Ok(values)
    }

    pub async fn all<D: DeserializeOwned>(&self, cf: &str) -> AppResult<Vec<D>> {
        let mut values = vec![];
        for (_, value) in self.db.all(Some(cf), None).await.map_err(to_err)? {
            values.push(from_b(value)?);
        }
        Ok(values)
    }
}

fn to_b<S: Serialize>(value: &S) -> AppResult<Vec<u8>> {
    postcard::to_allocvec(value).map_err(|e| AppError::Ser(e.to_string()))
}

fn from_b<D: DeserializeOwned>(bytes: Vec<u8>) -> AppResult<D> {
    postcard::from_bytes(&bytes).map_err(|e| AppError::Des(e.to_string()))
}

fn to_err(e: AsyncRocksError) -> AppError {
    tracing::error!("{}", e);
    AppError::DB
}
