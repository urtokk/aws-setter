use serde_derive::{
    Deserialize,
    Serialize
};

use color_eyre::Result;

use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug)]
pub struct AwsSetterConfig {
    pub email: String,
    profiles: HashMap<String,String>,
}

impl AwsSetterConfig {
    pub fn load(path: &str) -> Result<AwsSetterConfig> {
        let config = {
            let mut configger = config::Config::default();
            configger.merge(config::File::with_name(path)).unwrap();
            configger.try_into::<AwsSetterConfig>()?
        };

        Ok(config)
    }

    pub fn get_role(&self, profile: &str) -> Option<&String> {
        self.profiles.get_key_value(profile).map( |v| v.1)
    }

    pub fn list_profiles(&self) -> Vec<&String> {
        self.profiles.keys().collect()
    }
}
