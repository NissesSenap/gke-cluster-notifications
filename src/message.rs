pub mod attributes;
pub mod slack;

use base64::prelude::*;
use serde::{de, Deserialize, Deserializer};

use self::attributes::Attributes;

#[derive(Debug, Deserialize)]
pub struct PubSubMessage {
    pub message: Message,
    pub subscription: String,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct Message {
    pub attributes: Attributes,
    message_id: String,
    publish_time: String,

    #[serde(deserialize_with = "from_base64")]
    data: String,
}

impl Message {
    pub fn with_project_name(self, project_name: String) -> Self {
        Self { attributes: self.attributes.with_project_name(project_name), ..self }
    }

    pub fn fmt(&self) -> String {
        match self.attributes.log_message() {
            Ok(msg) => msg,
            Err(err) => {
                if self.data.is_empty() {
                    err
                } else {
                    format!("{}: {}", err, self.data)
                }
            }
        }
    }
}

fn from_base64<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    String::deserialize(deserializer)
        .and_then(|str| BASE64_STANDARD.decode(str).map_err(de::Error::custom))
        .map(String::from_utf8)
        .and_then(|res| res.map_err(de::Error::custom))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Deserialize)]
    pub struct TestCase {
        pub description: String,
        pub message_json: String,
        pub log_message: String,
        pub project_name: Option<String>,
    }

    pub fn test_cases() -> Vec<TestCase> {
        serde_yaml::from_slice(include_bytes!("message/tests.yaml")).unwrap()
    }

    #[test]
    fn deserialize() {
        for test in test_cases() {
            let mut message = match serde_json::from_str::<Message>(&test.message_json) {
                Ok(m) => m,
                Err(e) => panic!("{} failed: {e} {:#?}", test.description, test.message_json),
            };

            if let Some(project_name) = test.project_name {
                message = message.with_project_name(project_name.to_string());
            }

            println!("{message:#?}");
            assert_eq!(message.fmt(), test.log_message);
        }
    }
}
