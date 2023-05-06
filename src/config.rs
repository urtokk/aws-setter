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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_config() {
        let config = AwsSetterConfig::load("resources/test_config.toml").unwrap();
        assert_eq!(config.email, "test@example.com");
        assert_eq!(config.profiles.len(), 2);
        assert_eq!(config.get_role("default"), Some(&"arn:aws:iam::123456789012:role/MyRole".to_string()));
    }

    #[test]
    fn test_get_role() {
        let config = AwsSetterConfig {
            email: "test@example.com".to_string(),
            profiles: [("default".to_string(), "arn:aws:iam::123456789012:role/MyRole".to_string())]
                .iter().cloned().collect(),
        };
        assert_eq!(config.get_role("default"), Some(&"arn:aws:iam::123456789012:role/MyRole".to_string()));
        assert_eq!(config.get_role("nonexistent"), None);
    }

    #[test]
    fn test_list_profiles() {
        let config = AwsSetterConfig {
            email: "test@example.com".to_string(),
            profiles: [("default".to_string(), "arn:aws:iam::123456789012:role/MyRole".to_string()),
                       ("other".to_string(), "arn:aws:iam::123456789012:role/OtherRole".to_string())]
                .iter().cloned().collect(),
        };
        let profiles = config.list_profiles();
        assert_eq!(profiles.len(), 2);
        assert!(profiles.contains(&&"default".to_string()));
        assert!(profiles.contains(&&"other".to_string()));
    }
}