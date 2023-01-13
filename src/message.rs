mod attributes;

use base64::prelude::*;
use serde::{de, Deserialize, Deserializer, Serialize};

use self::attributes::Attributes;

#[derive(Debug, Deserialize, Serialize)]
pub struct PubSubMessage {
    pub message: Message,
    pub subscription: String,
}

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct Message {
    attributes: Attributes,
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
        match self.attributes.message() {
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

    struct MessageTestCase {
        description: &'static str,
        message_json: &'static str,
        log_message: &'static str,
        project_name: Option<&'static str>,
    }

    #[test]
    fn deserialize() {
        for test in MESSAGE_TEST_CASE {
            let mut message = match serde_json::from_str::<Message>(test.message_json) {
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

    const MESSAGE_TEST_CASE: [MessageTestCase; 14] = [
        // SecurityBulletinEvent
        MessageTestCase {
            description: "SecurityBulletinEvent:ControlPlane",
            message_json: r#"{
                "attributes": {
                    "project_id": "0123456789",
                    "payload": "{\"affectedSupportedMinors\":[\"1.18\",\"1.19\",\"1.20\",\"1.21\",\"1.22\",\"1.23\"],\"briefDescription\":\"A security vulnerability, CVE-2021-43527, has been discovered in any binary that links to the vulnerable versions of libnss3 found in Network Security Services (NSS) versions prior to 3.73 or 3.68.1.\",\"bulletinId\":\"GCP-2022-005\",\"bulletinUri\":\"https://cloud.google.com/kubernetes-engine/docs/security-bulletins#gcp-2022-005\",\"cveIds\":[\"CVE-2021-43527\"],\"patchedVersions\":[\"1.18.20-gke.6101\",\"1.19.16-gke.6100\",\"1.20.15-gke.200\",\"1.21.9-gke.200\",\"1.22.6-gke.600\",\"1.23.3-gke.500\"],\"resourceTypeAffected\":\"RESOURCE_TYPE_CONTROLPLANE\",\"severity\":\"Medium\",\"suggestedUpgradeTarget\":\"1.22.6-gke.1000\"}",
                    "cluster_location": "us-central1",
                    "type_url": "type.googleapis.com/google.container.v1beta1.SecurityBulletinEvent",
                    "cluster_name": "test-cluster"
                },
                "message_id": "1722065266338564",
                "publish_time": "2023-01-13T19:51:24.884Z",
                "data": "U2VjdXJpdHkgQnVsbGV0aW4gR0NQLTIwMjItMDA1IHRoYXQgYWZmZWN0cyB0aGlzIGNsdXN0ZXIgaGFzIGJlZW4gaXNzdWVk"
            }"#,
            log_message:  "Security bulletin GCP-2022-005 affecting projects/test-project/locations/us-central1/clusters/test-cluster has been issued",
            project_name: Some("test-project"),
        },
        MessageTestCase {
            description: "SecurityBulletinEvent:Node",
            message_json: r#"{
                "attributes": {
                    "payload": "{\"affectedSupportedMinors\":[\"1.18\",\"1.19\",\"1.20\",\"1.21\",\"1.22\",\"1.23\"],\"briefDescription\":\"A security vulnerability, CVE-2021-43527, has been discovered in any binary that links to the vulnerable versions of libnss3 found in Network Security Services (NSS) versions prior to 3.73 or 3.68.1.\",\"bulletinId\":\"GCP-2022-005\",\"bulletinUri\":\"https://cloud.google.com/kubernetes-engine/docs/security-bulletins#gcp-2022-005\",\"cveIds\":[\"CVE-2021-43527\"],\"patchedVersions\":[\"1.18.20-gke.6101\",\"1.19.16-gke.6100\",\"1.20.15-gke.200\",\"1.21.9-gke.200\",\"1.22.6-gke.600\",\"1.23.3-gke.500\"],\"resourceTypeAffected\":\"RESOURCE_TYPE_NODE\",\"severity\":\"Medium\",\"suggestedUpgradeTarget\":\"1.22.6-gke.1000\"}",
                    "cluster_name": "test-cluster",
                    "type_url": "type.googleapis.com/google.container.v1beta1.SecurityBulletinEvent",
                    "project_id": "0123456789",
                    "cluster_location": "us-central1"
                },
                "message_id": "5998008844325583",
                "publish_time": "2023-01-13T19:51:24.884Z",
                "data": "U2VjdXJpdHkgQnVsbGV0aW4gR0NQLTIwMjItMDA1IHRoYXQgYWZmZWN0cyB0aGlzIGNsdXN0ZXIgaGFzIGJlZW4gaXNzdWVk"
            }"#,
            log_message:  "Security bulletin GCP-2022-005 affecting projects/test-project/locations/us-central1/clusters/test-cluster has been issued",
            project_name: Some("test-project"),
        },
        MessageTestCase {
            description: "SecurityBulletinEvent:ControlPlane:NoSuggestedUpgradeTarget",
            message_json: r#"{
                "attributes": {
                    "cluster_location": "us-west1",
                    "cluster_name": "test-cluster",
                    "payload": "{\"resourceTypeAffected\":\"RESOURCE_TYPE_CONTROLPLANE\", \"bulletinId\":\"gcp-2022-008\", \"cveIds\":[\"CVE-2022-23606\", \"CVE-2022-21655\", \"CVE-2021-43826\", \"CVE-2021-43825\", \"CVE-2021-43824\", \"CVE-2022-21654\", \"CVE-2022-21657\", \"CVE-2022-21656\"], \"severity\":\"High\", \"bulletinUri\":\"https://cloud.google.com/kubernetes-engine/docs/security-bulletins#gcp-2022-008\", \"briefDescription\":\"The Envoy project recently discovered a set of vulnerabilities which may impact GKE clusters using Anthos Service Mesh, Istio-on-GKE, or custom Istio deployments. Refer to the security bulletin in the bulletinUri field of this notification and upgrade your Istio or ASM deployment to one of the fixed versions. If you do not use Istio or ASM, you are not affected by this bulletin.\", \"affectedSupportedMinors\":[\"1.18\", \"1.19\", \"1.20\", \"1.21\", \"1.22\", \"1.23\"], \"manualStepsRequired\":true}",
                    "project_id": "0123456789",
                    "type_url": "type.googleapis.com/google.container.v1beta1.SecurityBulletinEvent"
                },
                "message_id": "5782030237553958",
                "publish_time": "2023-01-13T19:51:24.884Z",
                "data": "U2VjdXJpdHkgQnVsbGV0aW4gZ2NwLTIwMjItMDA4IHRoYXQgYWZmZWN0cyB0aGlzIGNsdXN0ZXIgaGFzIGJlZW4gaXNzdWVk"
            }"#,
            log_message: "Security bulletin gcp-2022-008 affecting projects/0123456789/locations/us-west1/clusters/test-cluster has been issued",
            project_name: None
        },
        // UpgradeAvailableEvent
        MessageTestCase {
            description: "UpgradeAvailableEvent:ControlPlane",
            message_json: r#"{
                "attributes": {
                    "project_id": "0123456789",
                    "cluster_name": "test-cluster",
                    "cluster_location": "us-central1",
                    "payload": "{\"releaseChannel\":{\"channel\":\"RAPID\"},\"resourceType\":\"MASTER\",\"version\":\"1.22.6-gke.300\"}",
                    "type_url": "type.googleapis.com/google.container.v1beta1.UpgradeAvailableEvent"
                },
                "message_id": "9266639407169843",
                "publish_time": "2023-01-13T19:51:24.884Z",
                "data": "TmV3IG1hc3RlciB2ZXJzaW9uICIxLjIyLjYtZ2tlLjMwMCIgaXMgYXZhaWxhYmxlIGZvciB1cGdyYWRlIGluIHRoZSBSQVBJRCBjaGFubmVsLg=="
            }"#,
            log_message:  "Control plane projects/test-project/locations/us-central1/clusters/test-cluster has new version 1.22.6-gke.300 available for upgrade in the RAPID channel",
            project_name: Some("test-project"),
        },
        MessageTestCase {
            description: "UpgradeAvailableEvent:NodePool",
            message_json: r#"{
                "attributes": {
                    "cluster_location": "us-central1",
                    "project_id": "0123456789",
                    "type_url": "type.googleapis.com/google.container.v1beta1.UpgradeAvailableEvent",
                    "cluster_name": "test-cluster",
                    "payload": "{\"releaseChannel\":{\"channel\":\"RAPID\"},\"resource\":\"projects/test-project/locations/us-central1/clusters/test-cluster/nodePools/nap-e2-medium-ww57dx1i\",\"resourceType\":\"NODE_POOL\",\"version\":\"1.22.6-gke.300\"}"
                },
                "message_id": "6301543860902416",
                "publish_time": "2023-01-13T19:51:24.884Z",
                "data": "TmV3IG5vZGUgdmVyc2lvbiAiMS4yMi42LWdrZS4zMDAiIGlzIGF2YWlsYWJsZSBmb3IgdXBncmFkZSBpbiB0aGUgUkFQSUQgY2hhbm5lbC4="
            }"#,
            log_message:  "Node pool projects/test-project/locations/us-central1/clusters/test-cluster/nodePools/nap-e2-medium-ww57dx1i has new version 1.22.6-gke.300 available for upgrade in the RAPID channel",
            project_name: Some("test-project"),
        },
        MessageTestCase {
            description: "UpgradeAvailableEvent:UnknownResourceType",
            message_json: r#"{
                "attributes": {
                    "cluster_location": "us-central1",
                    "project_id": "0123456789",
                    "type_url": "type.googleapis.com/google.container.v1beta1.UpgradeAvailableEvent",
                    "cluster_name": "test-cluster",
                    "payload": "{\"releaseChannel\":{\"channel\":\"RAPID\"},\"resourceType\":\"SOME_TYPE\",\"version\":\"1.22.6-gke.300\"}"
                },
                "message_id": "2206175285646633",
                "publish_time": "2023-01-13T19:51:24.884Z",
                "data": "bG9yZW0gaXBzdW0="
            }"#,
            log_message:  "Unknown resource type `SOME_TYPE` encountered",
            project_name: Some("test-project"),
        },
        // UpgradeEvent
        MessageTestCase {
            description: "UpgradeEvent:ControlPlane",
            message_json: r#"{
                "attributes": {
                    "payload": "{\"currentVersion\":\"1.22.4-gke.1501\",\"operation\":\"operation-1646321640211-5bc1f505\",\"operationStartTime\":\"2022-03-03T15:34:00.211684830Z\",\"resourceType\":\"MASTER\",\"targetVersion\":\"1.22.6-gke.300\"}",
                    "project_id": "0123456789",
                    "cluster_name": "test-cluster",
                    "cluster_location": "us-central1",
                    "type_url": "type.googleapis.com/google.container.v1beta1.UpgradeEvent"
                },
                "message_id": "9800598855834432",
                "publish_time": "2023-01-13T19:51:24.884Z",
                "data": "TWFzdGVyIGlzIHVwZ3JhZGluZyB0byB2ZXJzaW9uIDEuMjIuNi1na2UuMzAwLg=="
            }"#,
            log_message: "Control plane projects/0123456789/locations/us-central1/clusters/test-cluster is upgrading from version 1.22.4-gke.1501 to 1.22.6-gke.300",
            project_name: None,
        },
        MessageTestCase {
            description: "UpgradeEvent:NodePool",
            message_json: r#"{
                "attributes": {
                    "payload": "{\"currentVersion\":\"1.22.4-gke.1501\",\"operation\":\"operation-1646323461754-a5e72991\",\"operationStartTime\":\"2022-03-03T16:04:21.754874604Z\",\"resource\":\"projects/test-project/locations/us-central1/clusters/test-cluster/nodePools/nap-e2-medium-ww57dx1i\",\"resourceType\":\"NODE_POOL\",\"targetVersion\":\"1.22.6-gke.300\"}",
                    "cluster_name": "test-cluster",
                    "type_url": "type.googleapis.com/google.container.v1beta1.UpgradeEvent",
                    "project_id": "0123456789",
                    "cluster_location": "us-central1"
                },
                "message_id": "8203570239583755",
                "publish_time": "2023-01-13T19:51:24.884Z",
                "data": "Tm9kZSBwb29sIHByb2plY3RzL3Rlc3QtcHJvamVjdC9sb2NhdGlvbnMvdXMtY2VudHJhbDEvY2x1c3RlcnMvdGVzdC1jbHVzdGVyL25vZGVQb29scy9uYXAtZTItbWVkaXVtLXd3NTdkeDFpIGlzIHVwZ3JhZGluZyB0byB2ZXJzaW9uIDEuMjIuNi1na2UuMzAwLg=="
            }"#,
            log_message: "Node pool projects/test-project/locations/us-central1/clusters/test-cluster/nodePools/nap-e2-medium-ww57dx1i is upgrading from 1.22.4-gke.1501 to 1.22.6-gke.300",
            project_name: None,
        },
        MessageTestCase {
            description: "UpgradeEvent:UnknownResourceType",
            message_json: r#"{
                "attributes": {
                    "payload": "{\"resourceType\":\"SOME_TYPE\",\"targetVersion\":\"1.22.6-gke.300\"}",
                    "cluster_name": "test-cluster",
                    "type_url": "type.googleapis.com/google.container.v1beta1.UpgradeEvent",
                    "project_id": "0123456789",
                    "cluster_location": "us-central1"
                },
                "message_id": "6663929498430716",
                "publish_time": "2023-01-13T19:51:24.884Z",
                "data": "Zm9vYmFy"
            }"#,
            log_message: "Unknown resource type `SOME_TYPE` encountered",
            project_name: None,
        },
        // UnknownEvent
        MessageTestCase {
            description: "UnknownEvent",
            message_json: r#"{
                "attributes": {
                    "payload": "{\"someField\":\"some value\",\"anotherField\":\"another value\",\"releaseChannel\":{\"channel\":\"REGULAR\"}}",
                    "project_id": "0123456789",
                    "cluster_name": "test-cluster",
                    "cluster_location": "us-central1",
                    "type_url": "type.googleapis.com/google.container.v1beta1.UnknownEvent"
                },
                "message_id": "4154633824166090",
                "publish_time": "2023-01-13T19:51:24.884Z",
                "data": "UmVjZWl2ZWQgdW5rbm93biBldmVudCBmb3IgdGhpcyBjbHVzdGVy"
            }"#,
            log_message: "Unknown message type `type.googleapis.com/google.container.v1beta1.UnknownEvent` encountered: Received unknown event for this cluster",
            project_name: None,
        },
        MessageTestCase {
            description: "UnknownEvent:EmptyPayload",
            message_json: r#"{
                "attributes": {
                    "payload": "{}",
                    "project_id": "0123456789",
                    "cluster_name": "test-cluster",
                    "cluster_location": "us-central1",
                    "type_url": "type.googleapis.com/google.container.v1beta1.UnknownEvent"
                },
                "message_id": "3958578237550302",
                "publish_time": "2023-01-13T19:51:24.884Z",
                "data": "VW5rbm93biBldmVudCB3aXRoIGVtcHR5IHBheWxvYWQ="
            }"#,
            log_message: "Unknown message type `type.googleapis.com/google.container.v1beta1.UnknownEvent` encountered: Unknown event with empty payload",
            project_name: None,
        },
        // InvalidMessage
        MessageTestCase {
            description: "InvalidMessage:Empty",
            message_json: r#"{}"#,
            log_message:  "Empty or invalid payload",
            project_name: None,
        },
        MessageTestCase {
            description: "InvalidMessage:MissingAttributes",
            message_json: r#"{"data": "bG9yZW0gaXBzdW0="}"#,
            log_message:  "Empty or invalid payload: lorem ipsum",
            project_name: None,
        },
        MessageTestCase {
            description: "InvalidMessage:MissingData",
            message_json: r#"{"attributes":{"type_url":"SOME_TYPE"}}"#,
            log_message:  "Empty or invalid payload",
            project_name: None,
        },
    ];
}
