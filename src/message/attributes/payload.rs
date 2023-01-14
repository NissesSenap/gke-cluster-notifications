use std::fmt::Display;

use serde::{de, Deserialize};

/// An object carrying notification-specific information.
#[derive(Debug, Default, Deserialize)]
pub enum Payload {
    SecurityBulletinEvent(SecurityBulletinEvent),
    UpgradeAvailableEvent(UpgradeAvailableEvent),
    UpgradeEvent(UpgradeEvent),
    UnknownType(String),

    #[default]
    None,
}

impl Payload {
    pub fn as_upgrade_available_event(&self) -> Option<&UpgradeAvailableEvent> {
        if let Self::UpgradeAvailableEvent(v) = self {
            Some(v)
        } else {
            None
        }
    }
}

impl Display for Payload {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Payload::SecurityBulletinEvent(_) => stringify!(SecurityBulletinEvent),
            Payload::UpgradeAvailableEvent(_) => stringify!(UpgradeAvailableEvent),
            Payload::UpgradeEvent(_) => stringify!(UpgradeEvent),
            Payload::UnknownType(_) => stringify!(UnknownType),
            Payload::None => stringify!(None),
        })
    }
}

/// SecurityBulletinEvent is a notification sent to customers when
/// a security bulletin has been posted that they are vulnerable to.
#[derive(Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct SecurityBulletinEvent {
    /// The GKE minor versions affected by this vulnerability.
    pub affected_supported_minors: Vec<String>,

    /// A brief description of the bulletin. See the bulletin pointed
    /// to by the bulletin_uri field for an expanded description.
    pub brief_description: String,

    /// The ID of the bulletin corresponding to the vulnerability.
    pub bulletin_id: String,

    /// The URI link to the bulletin on the website for more information.
    pub bulletin_uri: String,

    /// The CVEs associated with this bulletin.
    pub cve_ids: Vec<String>,

    /// If this field is specified, it means there are manual steps
    /// that the user must take to make their clusters safe.
    pub manual_steps_required: bool,

    /// The GKE versions where this vulnerability is patched.
    pub patched_versions: Vec<String>,

    /// The resource type (node/control plane) that has the vulnerability.
    /// Multiple notifications (1 notification per resource type) will be
    /// sent for a vulnerability that affects > 1 resource type.
    pub resource_type_affected: String,

    /// The severity of this bulletin as it relates to GKE.
    pub severity: String,

    /// This represents a version selected from the patched_versions
    /// field that the cluster receiving this notification should most
    /// likely want to upgrade to based on its current version. Note
    /// that if this notification is being received by a given cluster,
    /// it means that this version is currently available as an upgrade
    /// target in that cluster's location.
    pub suggested_upgrade_target: String,
}

impl SecurityBulletinEvent {
    pub fn resource_type_affected(&self) -> String {
        match self.resource_type_affected.as_str() {
            "RESOURCE_TYPE_CONTROLPLANE" => "Control Plane".to_string(),
            "RESOURCE_TYPE_NODE" => "Node".to_string(),
            rt => rt.to_lowercase().trim_start_matches("resource_type_").to_string(),
        }
    }

    pub fn manual_steps_required(&self) -> &str {
        match self.manual_steps_required {
            true => "Yes",
            false => "No",
        }
    }
}

/// UpgradeAvailableEvent is sent when a new available version is released.
#[derive(Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct UpgradeAvailableEvent {
    /// The release channel of the version.
    pub release_channel: ReleaseChannel,

    /// (Optional) Relative path to the resource. For
    /// example, the relative path of the node pool.
    pub resource: Option<String>,

    /// The resource type of the release version.
    pub resource_type: ResourceType,

    /// The release version available for upgrade.
    pub version: String,
}

impl UpgradeAvailableEvent {
    pub fn node_pool_name(&self) -> Option<String> {
        if let Some(resource) = &self.resource {
            if let Some((_, name)) = resource.split_once("nodePools/") {
                return Some(name.to_string());
            }
        }
        None
    }
}

/// Indicates which release channel a cluster is subscribed to.
#[derive(Debug, Default, Deserialize)]
#[serde(tag = "channel", rename_all = "UPPERCASE")]
pub enum ReleaseChannel {
    #[default]
    Unspecified,
    Rapid,
    Regular,
    Stable,
}

impl std::fmt::Display for ReleaseChannel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            ReleaseChannel::Unspecified => "UNSPECIFIED",
            ReleaseChannel::Rapid => "RAPID",
            ReleaseChannel::Regular => "REGULAR",
            ReleaseChannel::Stable => "STABLE",
        })
    }
}

/// UpgradeEvent is a notification sent when a resource is upgrading.
#[derive(Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct UpgradeEvent {
    /// The current version before the upgrade.
    pub current_version: String,

    /// The operation associated with this upgrade.
    pub operation: String,

    /// The time when the operation was started.
    pub operation_start_time: String,

    /// (Optional) Relative path to the resource. For example in
    /// node pool upgrades, the relative path of the node pool.
    pub resource: Option<String>,

    /// The resource type that is upgrading.
    pub resource_type: ResourceType,

    /// The target version for the upgrade.
    pub target_version: String,
}

impl UpgradeEvent {
    pub fn node_pool_name(&self) -> Option<String> {
        if let Some(resource) = &self.resource {
            if let Some((_, name)) = resource.split_once("nodePools/") {
                return Some(name.to_string());
            }
        }
        None
    }
}

#[derive(Debug)]
pub enum ResourceType {
    ControlPlane,
    NodePool,
    Unknown(String),
}

impl Default for ResourceType {
    fn default() -> Self {
        ResourceType::Unknown("UPGRADE_RESOURCE_TYPE_UNSPECIFIED".to_string())
    }
}

impl<'de> Deserialize<'de> for ResourceType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct ResourceTypeVisitor;
        impl<'de> de::Visitor<'de> for ResourceTypeVisitor {
            type Value = ResourceType;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("enum ResourceType")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(match v {
                    "MASTER" => ResourceType::ControlPlane,
                    "NODE_POOL" => ResourceType::NodePool,
                    _ => ResourceType::Unknown(v.to_string()),
                })
            }
        }

        deserializer.deserialize_identifier(ResourceTypeVisitor)
    }
}
