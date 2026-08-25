#[cfg(test)]
mod tests {
    use crate::data::config::IcvConfig;
    use config::{Config, File, FileFormat};
    use std::error::Error;

    #[test]
    fn test_deserialization() -> Result<(), Box<dyn Error>> {
        const CONFIG: &str = r#"
[communication]
backend = "mqtt"

[communication.mqtt]
host = "localhost"
port = 8003
client_id = "my_client_name"
username = "jdoe"
password = "mys4fepa$$word"
qos = 0
topic = "my/cool/machine"
tls = false
cert_path = "/path/to/cert.pem"
key_path = "/path/to/private_key.pem"

[communication.opcua]
host = "localhost"
port = 34645
username = "test"
password = "password"

[camera]
backend = "sick_sec"

[camera.sick_sec]
host = "172.16.0.1"
username = "Service"
password = "servicelevel""#;

        let settings = Config::builder()
            .add_source(File::from_str(CONFIG, FileFormat::Toml))
            .build()?;

        let config: IcvConfig = settings.try_deserialize::<IcvConfig>()?;

        Ok(())
    }
}
