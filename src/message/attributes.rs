mod payload;

use serde::{de, Deserialize, Serialize};

use self::payload::Payload;

const CONTROL_PLANE: &str = "MASTER";
const NODE_POOL: &str = "NODE_POOL";

#[derive(Debug, Default, Serialize)]
pub struct Attributes {
    project_id: String,
    project_name: Option<String>, // Unfortunately not included in the pub/sub message (filled from env var)
    cluster_name: String,
    cluster_location: String,
    type_url: String,
    payload: Payload,
}

impl Attributes {
    pub fn with_project_name(self, project_name: String) -> Self {
        Self { project_name: Some(project_name), ..self }
    }

    pub fn message(&self) -> Result<String, String> {
        match &self.payload {
            Payload::SecurityBulletinEvent(p) => Ok(format!(
                "Security bulletin {} affecting {} has been issued",
                p.bulletin_id,
                self.resource_uri()
            )),
            Payload::UpgradeAvailableEvent(p) => match p.resource_type.as_str() {
                CONTROL_PLANE => Ok(format!(
                    "Control plane {} has new version {} available for upgrade in the {} channel",
                    self.resource_uri(),
                    p.version,
                    p.release_channel,
                )),
                NODE_POOL => Ok(format!(
                    "Node pool {} has new version {} available for upgrade in the {} channel",
                    self.resource_uri(),
                    p.version,
                    p.release_channel,
                )),
                resource_type => Ok(format!("Unknown resource type `{}` encountered", resource_type)),
            },
            Payload::UpgradeEvent(p) => match p.resource_type.as_str() {
                CONTROL_PLANE => Ok(format!(
                    "Control plane {} is upgrading from version {} to {}",
                    self.resource_uri(),
                    p.current_version,
                    p.target_version
                )),
                NODE_POOL => Ok(format!(
                    "Node pool {} is upgrading from {} to {}",
                    self.resource_uri(),
                    p.current_version,
                    p.target_version
                )),
                resource_type => Ok(format!("Unknown resource type `{}` encountered", resource_type)),
            },
            Payload::UnknownType(_) => Err(format!("Unknown message type `{}` encountered", self.type_url)),
            Payload::None => Err("Empty or invalid payload".to_string()),
        }
    }

    fn resource_uri(&self) -> String {
        if let Some(resource) = match &self.payload {
            Payload::UpgradeAvailableEvent(p) => match p.resource_type.as_str() {
                NODE_POOL => &p.resource,
                _ => &None,
            },
            Payload::UpgradeEvent(p) => match p.resource_type.as_str() {
                NODE_POOL => &p.resource,
                _ => &None,
            },
            _ => &None,
        } {
            resource.clone()
        } else {
            format!(
                "projects/{}/locations/{}/clusters/{}",
                self.project_name.as_ref().unwrap_or(&self.project_id),
                self.cluster_location,
                self.cluster_name
            )
        }
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

        deserializer.deserialize_struct("Attributes", &[], AttributesVisitor)
    }
}
