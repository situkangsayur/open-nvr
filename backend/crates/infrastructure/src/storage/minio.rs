use async_trait::async_trait;
use open_nvr_domain::errors::DomainError;
use open_nvr_domain::ports::ObjectStorage;
use s3::bucket::Bucket;
use s3::creds::Credentials;
use s3::region::Region;
use tracing::info;

pub struct MinioStorage {
    bucket: Box<Bucket>,
}

impl MinioStorage {
    pub async fn new(
        endpoint: &str,
        access_key: &str,
        secret_key: &str,
        bucket_name: &str,
    ) -> Result<Self, DomainError> {
        let region = Region::Custom {
            region: "us-east-1".to_string(),
            endpoint: endpoint.to_string(),
        };

        let credentials = Credentials::new(Some(access_key), Some(secret_key), None, None, None)
            .map_err(|e| DomainError::Storage(format!("Invalid S3 credentials: {}", e)))?;

        let bucket = Bucket::new(bucket_name, region, credentials)
            .map_err(|e| DomainError::Storage(format!("Failed to create bucket handle: {}", e)))?
            .with_path_style();

        info!(
            bucket = bucket_name,
            endpoint = endpoint,
            "MinIO storage initialized"
        );

        Ok(Self { bucket })
    }
}

#[async_trait]
impl ObjectStorage for MinioStorage {
    async fn put_object(
        &self,
        key: &str,
        data: &[u8],
        content_type: &str,
    ) -> Result<(), DomainError> {
        self.bucket
            .put_object_with_content_type(key, data, content_type)
            .await
            .map_err(|e| DomainError::Storage(format!("Failed to put object: {}", e)))?;
        Ok(())
    }

    async fn get_object(&self, key: &str) -> Result<Vec<u8>, DomainError> {
        let response = self
            .bucket
            .get_object(key)
            .await
            .map_err(|e| DomainError::Storage(format!("Failed to get object: {}", e)))?;
        Ok(response.to_vec())
    }

    async fn delete_object(&self, key: &str) -> Result<(), DomainError> {
        self.bucket
            .delete_object(key)
            .await
            .map_err(|e| DomainError::Storage(format!("Failed to delete object: {}", e)))?;
        Ok(())
    }

    async fn presigned_url(&self, key: &str, expiry_secs: u64) -> Result<String, DomainError> {
        let url = self
            .bucket
            .presign_get(key, expiry_secs as u32, None)
            .await
            .map_err(|e| DomainError::Storage(format!("Failed to generate presigned URL: {}", e)))?;
        Ok(url)
    }

    async fn list_objects(&self, prefix: &str) -> Result<Vec<String>, DomainError> {
        let results = self
            .bucket
            .list(prefix.to_string(), None)
            .await
            .map_err(|e| DomainError::Storage(format!("Failed to list objects: {}", e)))?;

        let keys: Vec<String> = results
            .into_iter()
            .flat_map(|r| r.contents)
            .map(|obj| obj.key)
            .collect();

        Ok(keys)
    }

    async fn get_total_size(&self, prefix: &str) -> Result<u64, DomainError> {
        let results = self
            .bucket
            .list(prefix.to_string(), None)
            .await
            .map_err(|e| DomainError::Storage(format!("Failed to list objects for size: {}", e)))?;

        let total: u64 = results
            .into_iter()
            .flat_map(|r| r.contents)
            .map(|obj| obj.size)
            .sum();

        Ok(total)
    }
}
