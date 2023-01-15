use reqwest::StatusCode;
use serde::Serialize;
use serde_json::{json, Value};

use super::{attributes::payload::Payload, Message};

#[derive(Debug, Serialize)]
pub struct WebhookMessage {
    text: String,
    blocks: Vec<Value>,
}

impl WebhookMessage {
    pub async fn post(&self, webhook: String) -> Result<String, String> {
        let body = serde_json::to_string(self).map_err(|e| e.to_string())?;
        let resp = reqwest::Client::new()
            .post(webhook)
            .body(body)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        let status = resp.status();
        let text = resp.text().await.map_err(|e| e.to_string())?;

        match status {
            StatusCode::OK => Ok(text),
            _ => Err(text),
        }
    }

    fn blocks(message: &Message) -> Vec<Value> {
        let attr = &message.attributes;
        let mut result = vec![];

        result.push(json!({
            "type": "section",
            "text": { "type": "mrkdwn", "text": format_args!(":gear: {}", message.markdown()) },
        }));

        match &attr.payload {
            Payload::SecurityBulletinEvent(p) => {
                result.push(json!({
                    "type": "section",
                    "text": { "type": "mrkdwn", "text": format_args!("*Brief Description*\n{}", p.brief_description) },
                }));

                result.push(json!({
                    "type": "section",
                    "fields": [
                        { "type": "mrkdwn", "text": format_args!("*Affected Resource Type*\n{}", p.resource_type_affected()) },
                        { "type": "mrkdwn", "text": format_args!("*Manual Steps Required*\n{}", p.manual_steps_required()) },
                    ]
                }));

                result.push(json!({
                    "type": "section",
                    "fields": [
                        { "type": "mrkdwn", "text": format_args!("*Project*\n{}", attr.project_name()) },
                        { "type": "mrkdwn", "text": format_args!("*Severity*\n{}", p.severity) },
                    ]
                }));

                if !p.patched_versions.is_empty() || !p.suggested_upgrade_target.is_empty() {
                    result.push(json!({
                        "type": "section",
                        "fields": [
                            { "type": "mrkdwn", "text": format_args!("*Patched Versions*\n{}", p.patched_versions.join("\n")) },
                            { "type": "mrkdwn", "text": format_args!("*Suggested Upgrade Target*\n{}", p.suggested_upgrade_target) },
                        ]
                    }));
                }

                result.push(json!({
                    "type": "section",
                    "fields": [
                        { "type": "mrkdwn", "text": format_args!("*Cluster*\n{}", attr.resource_url()) },
                        { "type": "mrkdwn", "text": format_args!("*Security Bulletin*\n<{}|View Details>", p.bulletin_uri) },
                    ]
                }));
            }
            Payload::UpgradeAvailableEvent(p) => {
                result.push(json!({
                    "type": "section",
                    "fields": [
                        { "type": "mrkdwn", "text": format_args!("*Project*\n{}", attr.project_name()) },
                        { "type": "mrkdwn", "text": format_args!("*Version*\n{}", p.version) },
                    ]
                }));

                result.push(json!({
                    "type": "section",
                    "fields": [
                        { "type": "mrkdwn", "text": format_args!("*Resource*\n<{}|View in Console>", attr.resource_url()) },
                        { "type": "mrkdwn", "text": format_args!("*Release Channel*\n{}", p.release_channel) },
                    ]
                }));
            }
            Payload::UpgradeEvent(p) => {
                result.push(json!({
                    "type": "section",
                    "fields": [
                        { "type": "mrkdwn", "text": format_args!("*Project*\n{}", attr.project_name()) },
                        { "type": "mrkdwn", "text": format_args!("*Current Version*\n{}", p.current_version) },
                    ]
                }));

                result.push(json!({
                    "type": "section",
                    "fields": [
                        { "type": "mrkdwn", "text": format_args!("*Resource*\n<{}|View in Console>", attr.resource_url()) },
                        { "type": "mrkdwn", "text": format_args!("*Target Version*\n{}", p.target_version) },
                    ]
                }));
            }
            _ => {
                result.push(json!({
                    "type": "section",
                    "fields": [
                        { "type": "mrkdwn", "text": format_args!("*Project*\n{}", attr.project_name()) },
                        { "type": "mrkdwn", "text": format_args!("*Message*\n{}", message.data) },
                    ]
                }));
                result.push(json!({
                    "type": "section",
                    "fields": [
                        { "type": "mrkdwn", "text": format_args!("*Resource*\n<{}|View in Console>", attr.resource_url()) },
                        { "type": "mrkdwn", "text": format_args!("*TypeUrl*\n{}", attr.type_url) },
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

impl From<&Message> for WebhookMessage {
    fn from(message: &Message) -> Self {
        WebhookMessage {
            text: format!(":gear: {}", message.plain_text()),
            blocks: WebhookMessage::blocks(message),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::message::tests::test_messages;

    #[tokio::test]
    async fn post() {
        for test in test_messages() {
            if test.message.is_invalid() {
                continue;
            }
            let message: WebhookMessage = (&test.message).into();

            if let Ok(webhook) = std::env::var("SLACK_WEBHOOK") {
                message.post(webhook).await.unwrap();
            }

            // Print JSON usable in Block Kit Builder preview: https://app.slack.com/block-kit-builder/
            println!("{}\n", json!({ "blocks": message.blocks }));
        }
    }
}
