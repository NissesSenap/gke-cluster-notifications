use serde::Serialize;
use serde_json::{json, Value};

use super::{
    attributes::payload::{Payload, ResourceType},
    Message,
};

#[derive(Serialize)]
pub struct WebhookMessage {
    text: String,
    blocks: Vec<Value>,
}

impl From<&Message> for WebhookMessage {
    fn from(message: &Message) -> Self {
        WebhookMessage { text: message.slack_plain_text(), blocks: message.slack_blocks() }
    }
}

pub trait SlackMessage {
    fn slack_plain_text(&self) -> String;
    fn slack_block_text(&self) -> String;
    fn slack_blocks(&self) -> Vec<Value>;
}

impl SlackMessage for Message {
    fn slack_plain_text(&self) -> String {
        let attr = &self.attributes;
        match &attr.payload {
            Payload::SecurityBulletinEvent(p) => format!(
                ":gear: Security bulletin {} affecting {} has been issued",
                p.bulletin_id, attr.cluster_name
            ),
            Payload::UpgradeAvailableEvent(p) => match &p.resource_type {
                ResourceType::ControlPlane => format!(
                    ":gear: {} control plane has new version available {}",
                    attr.cluster_name, p.version,
                ),
                ResourceType::NodePool => format!(
                    ":gear: {} node pool {} has new version available {}",
                    attr.cluster_name,
                    p.node_pool_name().unwrap_or_default(),
                    p.version,
                ),
                ResourceType::Unknown(str) => {
                    format!(":gear: {} unknown resource type {str}", attr.cluster_name)
                }
            },
            Payload::UpgradeEvent(p) => match &p.resource_type {
                ResourceType::ControlPlane => format!(
                    ":gear: {} control plane is upgrading to version {}",
                    attr.cluster_name, p.target_version
                ),
                ResourceType::NodePool => format!(
                    ":gear: {} node pool {} is upgrading to version {}",
                    attr.cluster_name,
                    p.node_pool_name().unwrap_or_default(),
                    p.target_version
                ),
                ResourceType::Unknown(str) => {
                    format!(":gear: {} unknown resource type {str}", attr.cluster_name)
                }
            },
            Payload::UnknownType(_) => {
                format!(":gear: {} received event of unknown type", attr.cluster_name)
            }
            Payload::None => ":gear: empty or invalid payload".to_string(),
        }
    }

    fn slack_block_text(&self) -> String {
        let attr = &self.attributes;
        match &attr.payload {
            Payload::SecurityBulletinEvent(p) => format!(
                ":gear: Security bulletin `{}` affecting `{}` has been issued",
                p.bulletin_id, attr.cluster_name
            ),
            Payload::UpgradeAvailableEvent(p) => match &p.resource_type {
                ResourceType::ControlPlane => format!(
                    ":gear: *`{}`* control plane has new version available",
                    attr.cluster_name,
                ),
                ResourceType::NodePool => format!(
                    ":gear: *`{}`* node pool `{}` has new version available",
                    attr.cluster_name,
                    p.node_pool_name().unwrap_or_default(),
                ),
                ResourceType::Unknown(str) => format!(
                    ":gear: *`{}`* unknown resource type `{str}` encountered on `{}`",
                    attr.cluster_name, attr.payload
                ),
            },
            Payload::UpgradeEvent(p) => match &p.resource_type {
                ResourceType::ControlPlane => {
                    format!(":gear: *`{}`* control plane is upgrading", attr.cluster_name)
                }
                ResourceType::NodePool => format!(
                    ":gear: *`{}`* node pool `{}` is upgrading",
                    attr.cluster_name,
                    p.node_pool_name().unwrap_or_default(),
                ),
                ResourceType::Unknown(str) => format!(
                    ":gear: *`{}`* unknown resource type `{str}` encountered on `{}`",
                    attr.cluster_name, attr.payload
                ),
            },
            Payload::UnknownType(_) => {
                format!(":gear: `{}` received event of unknown type", attr.cluster_name)
            }
            Payload::None => ":gear: empty or invalid payload".to_string(),
        }
    }

    fn slack_blocks(&self) -> Vec<Value> {
        let attr = &self.attributes;
        let mut result = vec![];

        result.push(json!({
            "type": "section",
            "text": { "type": "mrkdwn", "text": self.slack_block_text() },
        }));

        match &attr.payload {
            Payload::SecurityBulletinEvent(p) => {
                result.push(json!({
                    "type": "section",
                    "text": { "type": "mrkdwn", "text": format!("*Brief Description*\n{}", p.brief_description) },
                }));

                result.push(json!({
                    "type": "section",
                    "fields": [
                        { "type": "mrkdwn", "text": format!("*Affected Resource Type*\n{}", p.resource_type_affected()) },
                        { "type": "mrkdwn", "text": format!("*Manual Steps Required*\n{}", p.manual_steps_required()) },
                    ]
                }));

                result.push(json!({
                    "type": "section",
                    "fields": [
                        { "type": "mrkdwn", "text": format!("*Project*\n{}", attr.project_name()) },
                        { "type": "mrkdwn", "text": format!("*Severity*\n{}", p.severity) },
                    ]
                }));

                if !p.patched_versions.is_empty() || !p.suggested_upgrade_target.is_empty() {
                    result.push(json!({
                        "type": "section",
                        "fields": [
                            { "type": "mrkdwn", "text": format!("*Patched Versions*\n{}", p.patched_versions.join("\n")) },
                            { "type": "mrkdwn", "text": format!("*Suggested Upgrade Target*\n{}", p.suggested_upgrade_target) },
                        ]
                    }));
                }

                result.push(json!({
                    "type": "section",
                    "fields": [
                        { "type": "mrkdwn", "text": format!("*Cluster*\n{}", attr.resource_url()) },
                        { "type": "mrkdwn", "text": format!("*Security Bulletin*\n<{}|View Details>", p.bulletin_uri) },
                    ]
                }));
            }
            Payload::UpgradeAvailableEvent(p) => {
                result.push(json!({
                    "type": "section",
                    "fields": [
                        { "type": "mrkdwn", "text": format!("*Project*\n{}", attr.project_name()) },
                        { "type": "mrkdwn", "text": format!("*Version*\n{}", p.version) },
                    ]
                }));

                result.push(json!({
                    "type": "section",
                    "fields": [
                        { "type": "mrkdwn", "text": format!("*Resource*\n<{}|View in Console>", attr.resource_url()) },
                        { "type": "mrkdwn", "text": format!("*Release Channel*\n{}", p.release_channel) },
                    ]
                }));
            }
            Payload::UpgradeEvent(p) => {
                result.push(json!({
                    "type": "section",
                    "fields": [
                        { "type": "mrkdwn", "text": format!("*Project*\n{}", attr.project_name()) },
                        { "type": "mrkdwn", "text": format!("*Current Version*\n{}", p.current_version) },
                    ]
                }));

                result.push(json!({
                    "type": "section",
                    "fields": [
                        { "type": "mrkdwn", "text": format!("*Resource*\n<{}|View in Console>", attr.resource_url()) },
                        { "type": "mrkdwn", "text": format!("*Target Version*\n{}", p.target_version) },
                    ]
                }));
            }
            _ => {
                result.push(json!({
                    "type": "section",
                    "fields": [
                        { "type": "mrkdwn", "text": format!("*Project*\n{}", attr.project_name()) },
                        { "type": "mrkdwn", "text": format!("*Message*\n{}", self.data) },
                    ]
                }));
                result.push(json!({
                    "type": "section",
                    "fields": [
                        { "type": "mrkdwn", "text": format!("*Resource*\n<{}|View in Console>", attr.resource_url()) },
                        { "type": "mrkdwn", "text": format!("*TypeUrl*\n{}", attr.type_url) },
                    ]
                }));
            }
        };

        result.push(json!({
            "type": "context",
            "elements": [
                { "type": "mrkdwn", "text": attr.resource_uri() }
            ]
        }));

        result
    }
}
