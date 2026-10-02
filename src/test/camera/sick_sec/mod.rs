#[cfg(test)]
mod tests {
    use crate::camera::interface::CameraInterface;
    use crate::camera::sick_sec::connection::Sec;
    use crate::data::config::SickSecConfig;
    use serde_json::json;
    use std::error::Error;
    use std::fs;

    #[tokio::test]
    async fn test_auth() -> Result<(), Box<dyn Error>> {
        let sec_config_json = json!({
            "host": "172.31.45.146".to_string(),
            "username": "Service".to_string(),
            "password": "servicelevel".to_string()
        });

        let sec_config: SickSecConfig = serde_json::from_value(sec_config_json)?;

        let sec_client = Sec::new(sec_config).await?;

        sec_client.check_credentials().await?;

        Ok(())
    }

    #[tokio::test]
    async fn test_capture_image() -> Result<(), Box<dyn Error>> {
        let sec_config_json = json!({
            "host": "172.31.45.146".to_string(),
            "username": "Service".to_string(),
            "password": "servicelevel".to_string()
        });

        let sec_config: SickSecConfig = serde_json::from_value(sec_config_json)?;

        let sec_client = Sec::new(sec_config).await?;

        let image = sec_client.capture_image().await?;

        fs::write("testimg.jpeg", image)?;

        Ok(())
    }
}
