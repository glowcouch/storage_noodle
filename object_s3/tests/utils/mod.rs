pub struct MinioGaurd(pub std::process::Child);

impl MinioGaurd {
    pub async fn new(
        address: impl core::fmt::Display,
    ) -> Result<Self, Box<dyn core::error::Error>> {
        let child = std::process::Command::new("minio")
            .arg("server")
            .arg(format!("/tmp/{}", rand::random::<u64>()))
            .spawn()?;

        let wait_for_minio = async {
            loop {
                if let Ok(result) = reqwest::get(format!("{address}/minio/health/live")).await
                    && result.status().is_success()
                {
                    break;
                }

                tokio::time::sleep(core::time::Duration::from_millis(100)).await;
            }
        };
        tokio::time::timeout(core::time::Duration::from_secs(10), wait_for_minio)
            .await
            .map_err(|e| format!("Timed out waiting for minio: {e}"))?;

        Ok(MinioGaurd(child))
    }
}

impl Drop for MinioGaurd {
    fn drop(&mut self) {
        self.0.kill().unwrap();
        self.0.wait().unwrap();
    }
}
