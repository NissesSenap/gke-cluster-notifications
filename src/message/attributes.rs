pub mod payload;

use serde::{de, Deserialize};

use self::payload::{Payload, ResourceType};

#[derive(Debug, Default)]
pub struct Attributes {
    pub project_id: String,
    pub project_name: Option<String>, // Unfortunately not included in the pub/sub message (filled from env var)
    pub cluster_name: String,
    pub cluster_location: String,
    pub type_url: String,
    pub payload: Payload,
}

impl Attributes {
    pub fn with_project_name(self, project_name: String) -> Self {
        Self { project_name: Some(project_name), ..self }
    }

    pub fn log_message(&self) -> Result<String, String> {
        match &self.payload {
            Payload::SecurityBulletinEvent(p) => Ok(format!(
                "Security bulletin {} affecting {} has been issued",
                p.bulletin_id,
                self.resource_uri()
            )),
            Payload::UpgradeAvailableEvent(p) => match &p.resource_type {
                ResourceType::ControlPlane => Ok(format!(
                    "Control plane {} has new version {} available for upgrade in the {} channel",
                    self.resource_uri(),
                    p.version,
                    p.release_channel,
                )),
                ResourceType::NodePool => Ok(format!(
                    "Node pool {} has new version {} available for upgrade in the {} channel",
                    self.resource_uri(),
                    p.version,
                    p.release_channel,
                )),
                ResourceType::Unknown(str) => {
                    Ok(format!("Unknown resource type `{str}` encountered"))
                }
            },
            Payload::UpgradeEvent(p) => match &p.resource_type {
                ResourceType::ControlPlane => Ok(format!(
                    "Control plane {} is upgrading from version {} to {}",
                    self.resource_uri(),
                    p.current_version,
                    p.target_version
                )),
                ResourceType::NodePool => Ok(format!(
                    "Node pool {} is upgrading from {} to {}",
                    self.resource_uri(),
                    p.current_version,
                    p.target_version
                )),
                ResourceType::Unknown(str) => {
                    Ok(format!("Unknown resource type `{str}` encountered"))
                }
            },
            Payload::UnknownType(_) => {
                Err(format!("Unknown message type `{}` encountered", self.type_url))
            }
            Payload::None => Err("Empty or invalid payload".to_string()),
        }
    }

    pub fn project_name(&self) -> String {
        self.project_name.as_ref().unwrap_or(&self.project_id).to_string()
    }

    pub fn resource_uri(&self) -> String {
        if let Some(resource) = match &self.payload {
            Payload::UpgradeAvailableEvent(p) => match p.resource_type {
                ResourceType::NodePool => &p.resource,
                _ => &None,
            },
            Payload::UpgradeEvent(p) => match p.resource_type {
                ResourceType::NodePool => &p.resource,
                _ => &None,
            },
            _ => &None,
        } {
            resource.clone()
        } else {
            format!(
                "projects/{}/locations/{}/clusters/{}",
                self.project_name(),
                self.cluster_location,
                self.cluster_name
            )
        }
    }

    pub fn resource_url(&self) -> String {
        if let Some(node_pool_name) = match &self.payload {
            Payload::UpgradeAvailableEvent(p) => match p.resource_type {
                ResourceType::NodePool => p.node_pool_name(),
                _ => None,
            },
            Payload::UpgradeEvent(p) => match p.resource_type {
                ResourceType::NodePool => p.node_pool_name(),
                _ => None,
            },
            _ => None,
        } {
            format!(
                "https://console.cloud.google.com/kubernetes/nodepool/{}/{}/{}?project={}",
                self.cluster_location,
                self.cluster_name,
                node_pool_name,
                self.project_name(),
            )
        } else {
            format!(
                "https://console.cloud.google.com/kubernetes/clusters/details/{}/{}?project={}",
                self.cluster_location,
                self.cluster_name,
                self.project_name(),
            )
        }
    }

    pub fn is_node_pool_upgrade_available_event(&self) -> bool {
        self.payload
            .as_upgrade_available_event()
            .map(|p| matches!(p.resource_type, ResourceType::NodePool))
            .unwrap_or_default()
    }
}

impl<'de> Deserialize<'de> for Attributes {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(field_identifier, rename_all = "snake_case")]
        enum Field {
            ProjectId,
            ClusterName,
            ClusterLocation,
            TypeUrl,
            Payload,
        }

        struct AttributesVisitor;
        impl<'de> de::Visitor<'de> for AttributesVisitor {
            type Value = Attributes;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("struct Attributes")
            }

            fn visit_map<V>(self, mut map: V) -> Result<Attributes, V::Error>
            where
                V: de::MapAccess<'de>,
            {
                let mut project_id = None;
                let mut cluster_name = None;
                let mut cluster_location = None;
                let mut type_url = None;
                let mut payload = None;

                while let Some(field) = map.next_key()? {
                    match field {
                        Field::ProjectId => {
                            project_id = Some(map.next_value::<String>()?);
                        }
                        Field::ClusterName => {
                            cluster_name = Some(map.next_value::<String>()?);
                        }
                        Field::ClusterLocation => {
                            cluster_location = Some(map.next_value::<String>()?);
                        }
                        Field::TypeUrl => {
                            type_url = Some(map.next_value::<String>()?);
                        }
                        Field::Payload => {
                            payload = Some(map.next_value::<String>()?);
                        }
                    }
                }

                let project_id = project_id.unwrap_or_default();
                let project_name = None; // Pub/Sub message never includes the project_name
                let cluster_name = cluster_name.unwrap_or_default();
                let cluster_location = cluster_location.unwrap_or_default();
                let type_url = type_url.unwrap_or_default();
                let payload = payload.unwrap_or_default();

                let payload = match type_url.as_str() {
                    "type.googleapis.com/google.container.v1beta1.SecurityBulletinEvent" => {
                        Payload::SecurityBulletinEvent(
                            serde_json::from_str(&payload).map_err(de::Error::custom)?,
                        )
                    }
                    "type.googleapis.com/google.container.v1beta1.UpgradeAvailableEvent" => {
                        Payload::UpgradeAvailableEvent(
                            serde_json::from_str(&payload).map_err(de::Error::custom)?,
                        )
                    }
                    "type.googleapis.com/google.container.v1beta1.UpgradeEvent" => {
                        Payload::UpgradeEvent(
                            serde_json::from_str(&payload).map_err(de::Error::custom)?,
                        )
                    }
                    _ => {
                        if payload.is_empty() {
                            Payload::None
                        } else {
                            Payload::UnknownType(payload)
                        }
                    }
                };

                Ok(Attributes {
                    project_id,
                    project_name,
                    cluster_name,
                    cluster_location,
                    type_url,
                    payload,
                })
            }
        }

        deserializer.deserialize_map(AttributesVisitor)
    }
}
