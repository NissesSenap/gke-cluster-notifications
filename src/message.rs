pub mod attributes;
pub mod slack;

use base64::prelude::*;
use serde::{de, Deserialize, Deserializer};

use self::attributes::payload::{Payload, ResourceType};
use self::attributes::Attributes;

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
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

    pub fn is_invalid(&self) -> bool {
        self.data.is_empty() || self.attributes.is_invalid()
    }

    pub fn log_entry(&self) -> String {
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

    fn plain_text(&self) -> String {
        let attr = &self.attributes;
        match &attr.payload {
            Payload::SecurityBulletinEvent(p) => format!(
                "Security bulletin {} affecting {} has been issued",
                p.bulletin_id, attr.cluster_name
            ),
            Payload::UpgradeAvailableEvent(p) => match &p.resource_type {
                ResourceType::ControlPlane => format!(
                    "{} control plane has new version available {}",
                    attr.cluster_name, p.version,
                ),
                ResourceType::NodePool => format!(
                    "{} node pool {} has new version available {}",
                    attr.cluster_name,
                    p.node_pool_name().unwrap_or_default(),
                    p.version,
                ),
                ResourceType::Unknown(str) => {
                    format!("{} unknown resource type {str}", attr.cluster_name)
                }
            },
            Payload::UpgradeEvent(p) => match &p.resource_type {
                ResourceType::ControlPlane => format!(
                    "{} control plane is upgrading to version {}",
                    attr.cluster_name, p.target_version
                ),
                ResourceType::NodePool => format!(
                    "{} node pool {} is upgrading to version {}",
                    attr.cluster_name,
                    p.node_pool_name().unwrap_or_default(),
                    p.target_version
                ),
                ResourceType::Unknown(str) => {
                    format!("{} unknown resource type {str}", attr.cluster_name)
                }
            },
            _ if self.is_invalid() => "empty or invalid payload".to_string(),
            _ => format!("{} received event of unknown type", attr.cluster_name),
        }
    }

    fn markdown(&self) -> String {
        let attr = &self.attributes;
        match &attr.payload {
            Payload::SecurityBulletinEvent(p) => format!(
                "Security bulletin `{}` affecting `{}` has been issued",
                p.bulletin_id, attr.cluster_name
            ),
            Payload::UpgradeAvailableEvent(p) => match &p.resource_type {
                ResourceType::ControlPlane => {
                    format!("*`{}`* control plane has new version available", attr.cluster_name,)
                }
                ResourceType::NodePool => format!(
                    "*`{}`* node pool `{}` has new version available",
                    attr.cluster_name,
                    p.node_pool_name().unwrap_or_default(),
                ),
                ResourceType::Unknown(str) => format!(
                    "*`{}`* unknown resource type `{str}` encountered on `{}`",
                    attr.cluster_name, attr.payload
                ),
            },
            Payload::UpgradeEvent(p) => match &p.resource_type {
                ResourceType::ControlPlane => {
                    format!("*`{}`* control plane is upgrading", attr.cluster_name)
                }
                ResourceType::NodePool => format!(
                    "*`{}`* node pool `{}` is upgrading",
                    attr.cluster_name,
                    p.node_pool_name().unwrap_or_default(),
                ),
                ResourceType::Unknown(str) => format!(
                    "*`{}`* unknown resource type `{str}` encountered on `{}`",
                    attr.cluster_name, attr.payload
                ),
            },
            _ if self.is_invalid() => "empty or invalid payload".to_string(),
            _ => format!("`{}` received event of unknown type", attr.cluster_name),
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
    use std::collections::HashMap;

    pub struct TestCase {
        pub message: Message,
        log_entry: String,
        plain_text: String,
        markdown: String,
    }

    pub fn test_messages() -> Vec<TestCase> {
        serde_yaml::from_slice::<Vec<HashMap<String, String>>>(include_bytes!("message/tests.yaml"))
            .unwrap()
            .iter()
            .map(|fields| {
                let message_name = fields.get("name").unwrap().to_string();
                let project_name = fields.get("project_name").map(String::to_string);

                let message =
                    serde_json::from_str::<Message>(&fields.get("message").unwrap().to_string())
                        .map_err(|err| format!("invalid message {message_name}: {err}"))
                        .unwrap();

                TestCase {
                    message: match &project_name {
                        Some(name) => message.with_project_name(name.to_string()),
                        _ => message,
                    },
                    log_entry: fields.get("log_entry").unwrap().to_string(),
                    plain_text: fields.get("plain_text").unwrap().to_string(),
                    markdown: fields.get("markdown").unwrap().to_string(),
                }
            })
            .collect()
    }

    #[test]
    fn messages() {
        for test in test_messages() {
            println!("{:#?}", test.message);
        }
    }

    #[test]
    fn log_entry() {
        for test in test_messages() {
            let log_entry = test.message.log_entry();
            println!("{log_entry}");
            assert_eq!(log_entry, test.log_entry);
        }
    }

    #[test]
    fn plain_text() {
        for test in test_messages() {
            let plain_text = test.message.plain_text();
            println!("{plain_text}");
            assert_eq!(plain_text, test.plain_text);
        }
    }

    #[test]
    fn markdown() {
        for test in test_messages() {
            let markdown = test.message.markdown();
            println!("{markdown}");
            assert_eq!(markdown, test.markdown);
        }
    }
}
