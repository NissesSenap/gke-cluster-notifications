mod payload;

use serde::{de, Deserialize, Serialize};

use self::payload::Payload;

#[derive(Debug, Serialize)]
pub struct Attributes {
    project_id: String,
    cluster_name: String,
    cluster_location: String,
    type_url: String,
    payload: Payload,
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

                let project_id =
                    project_id.ok_or_else(|| de::Error::missing_field("project_id"))?;
                let cluster_name =
                    cluster_name.ok_or_else(|| de::Error::missing_field("cluster_name"))?;
                let cluster_location =
                    cluster_location.ok_or_else(|| de::Error::missing_field("cluster_location"))?;
                let type_url = type_url.ok_or_else(|| de::Error::missing_field("type_url"))?;
                let payload = payload.ok_or_else(|| de::Error::missing_field("payload"))?;

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
                    _ => Payload::UnknownEvent(
                        serde_json::from_str(&payload).map_err(de::Error::custom)?,
                    ),
                };

                Ok(Attributes { project_id, cluster_name, cluster_location, type_url, payload })
            }
        }

        deserializer.deserialize_struct("Attributes", &[], AttributesVisitor)
    }
}
