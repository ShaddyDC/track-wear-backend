use config::{Config, Environment};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub image_folder: String,
    pub google_client_id: String,
}

impl Settings {
    pub fn new() -> Self {
        Config::builder()
            .add_source(Environment::default())
            .build()
            .unwrap()
            .try_deserialize()
            .unwrap()
    }
}
