use serde::{Deserialize, Serialize};

/// An object carrying notification-specific information.
#[derive(Debug, Default, Deserialize, Serialize)]
pub enum Payload {
    SecurityBulletinEvent(SecurityBulletinEvent),
    UpgradeAvailableEvent(UpgradeAvailableEvent),
    UpgradeEvent(UpgradeEvent),
    UnknownType(String),

    #[default]
    None,
}

/// SecurityBulletinEvent is a notification sent to customers when
/// a security bulletin has been posted that they are vulnerable to.
#[derive(Debug, Default, Deserialize, Serialize)]
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

/// UpgradeAvailableEvent is sent when a new available version is released.
#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(default, rename_all = "camelCase")]
pub struct UpgradeAvailableEvent {
    /// The release channel of the version.
    pub release_channel: ReleaseChannel,

    /// (Optional) Relative path to the resource. For
    /// example, the relative path of the node pool.
    pub resource: Option<String>,

    /// The resource type of the release version.
    pub resource_type: String,

    /// The release version available for upgrade.
    pub version: String,
}

/// Indicates which release channel a cluster is subscribed to.
#[derive(Debug, Default, Deserialize, Serialize)]
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
#[derive(Debug, Default, Deserialize, Serialize)]
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
    pub resource_type: String,

    /// The target version for the upgrade.
    pub target_version: String,
}
