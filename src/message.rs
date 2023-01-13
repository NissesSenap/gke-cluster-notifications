mod attributes;

use base64::prelude::*;
use serde::{de, Deserialize, Deserializer, Serialize};

use self::attributes::Attributes;

#[derive(Debug, Deserialize, Serialize)]
pub struct PubSubMessage {
    message: Message,
    subscription: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct Message {
    attributes: Attributes,
    message_id: String,
    publish_time: String,

    #[serde(deserialize_with = "from_base64")]
    data: String,
}

fn from_base64<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    String::deserialize(deserializer)
        .and_then(|str| BASE64_STANDARD.decode(&str).map_err(|e| de::Error::custom(e)))
        .map(|bytes| String::from_utf8(bytes))
        .and_then(|res| res.or_else(|e| Err(de::Error::custom(e))))
}
