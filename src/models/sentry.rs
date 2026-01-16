use serde::{Deserialize, Deserializer, Serialize};

/// Sentry webhook payload for event_alert
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SentryWebhookPayload {
    pub action: String,
    pub data: SentryData,
    #[serde(default)]
    pub actor: Option<SentryActor>,
}

fn deserialize_id_option<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    use serde::de::Error;
    let v: serde_json::Value = Deserialize::deserialize(deserializer)?;
    match v {
        serde_json::Value::String(s) => Ok(Some(s)),
        serde_json::Value::Number(n) => Ok(Some(n.to_string())),
        serde_json::Value::Null => Ok(None),
        _ => Err(D::Error::custom("expected a string or a number")),
    }
}

fn deserialize_timestamp_option<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let v: serde_json::Value = Deserialize::deserialize(deserializer)?;
    match v {
        serde_json::Value::String(s) => Ok(Some(s)),
        serde_json::Value::Number(n) => {
            if let Some(f) = n.as_f64() {
                Ok(Some(f.to_string()))
            } else if let Some(i) = n.as_i64() {
                Ok(Some(i.to_string()))
            } else {
                Ok(Some(n.to_string()))
            }
        }
        serde_json::Value::Null => Ok(None),
        _ => Ok(None),
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SentryData {
    pub event: Option<SentryEvent>,
    pub triggered_rule: Option<String>,
    pub issue_alert: Option<SentryIssueAlert>,
    // For metric alerts
    pub metric_alert: Option<SentryMetricAlert>,
    pub description_text: Option<String>,
    pub description_title: Option<String>,
    pub web_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SentryEvent {
    #[serde(deserialize_with = "deserialize_id_option", default)]
    pub event_id: Option<String>,
    pub url: Option<String>,
    pub web_url: Option<String>,
    pub issue_url: Option<String>,
    #[serde(deserialize_with = "deserialize_id_option", default)]
    pub issue_id: Option<String>,
    #[serde(deserialize_with = "deserialize_id_option", default)]
    pub project: Option<String>,
    pub project_slug: Option<String>,
    pub project_name: Option<String>,
    #[serde(default)]
    pub environment: Option<String>,
    #[serde(rename = "type")]
    pub event_type: Option<String>,
    pub message: Option<String>,
    pub title: Option<String>,
    pub culprit: Option<String>,
    #[serde(deserialize_with = "deserialize_timestamp_option", default)]
    pub timestamp: Option<String>,
    pub datetime: Option<String>,
    pub platform: Option<String>,
    pub level: Option<String>,
    #[serde(default)]
    pub tags: Vec<(String, String)>,
    pub user: Option<SentryUser>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SentryUser {
    pub id: Option<String>,
    pub username: Option<String>,
    pub email: Option<String>,
    pub ip_address: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SentryIssueAlert {
    pub title: Option<String>,
    #[serde(default)]
    pub settings: Vec<SentryAlertSetting>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SentryAlertSetting {
    pub name: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SentryMetricAlert {
    pub title: Option<String>,
    pub alert_rule: Option<SentryAlertRule>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SentryAlertRule {
    pub id: Option<i64>,
    pub name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SentryActor {
    #[serde(rename = "type")]
    pub actor_type: Option<String>,
    #[serde(deserialize_with = "deserialize_id_option", default)]
    pub id: Option<String>,
    pub name: Option<String>,
}

impl SentryWebhookPayload {
    /// Extract project identifier from the payload
    pub fn get_project(&self) -> Option<String> {
        self.data
            .event
            .as_ref()
            .and_then(|e| {
                e.project
                    .clone()
                    .or_else(|| e.project_slug.clone())
                    .or_else(|| e.project_name.clone())
            })
            .or_else(|| {
                self.data.web_url.as_ref().and_then(|url| {
                    // Try to extract project slug from URL if possible
                    // e.g. https://sentry.io/organizations/org/issues/?project=123
                    if let Some(pos) = url.find("project=") {
                        let project_part = &url[pos + 8..];
                        let end_pos = project_part.find('&').unwrap_or(project_part.len());
                        Some(project_part[..end_pos].to_string())
                    } else {
                        None
                    }
                })
            })
    }

    /// Extract environment from the payload
    pub fn get_environment(&self) -> Option<String> {
        self.data.event.as_ref().and_then(|e| e.environment.clone())
    }

    /// Extract error level from the payload
    pub fn get_level(&self) -> Option<String> {
        self.data.event.as_ref().and_then(|e| e.level.clone())
    }

    /// Extract title/error message from the payload
    pub fn get_title(&self) -> Option<String> {
        self.data
            .description_title
            .clone()
            .or_else(|| self.data.event.as_ref().and_then(|e| e.title.clone()))
            .or_else(|| self.data.issue_alert.as_ref().and_then(|a| a.title.clone()))
            .or_else(|| {
                self.data
                    .metric_alert
                    .as_ref()
                    .and_then(|a| a.title.clone())
            })
    }

    /// Extract detailed message from the payload
    pub fn get_message(&self) -> Option<String> {
        self.data
            .description_text
            .clone()
            .or_else(|| self.data.event.as_ref().and_then(|e| e.message.clone()))
    }

    /// Extract web URL for the issue
    pub fn get_web_url(&self) -> Option<String> {
        self.data
            .web_url
            .clone()
            .or_else(|| self.data.event.as_ref().and_then(|e| e.web_url.clone()))
    }

    /// Extract triggered rule name
    pub fn get_triggered_rule(&self) -> Option<String> {
        self.data.triggered_rule.clone()
    }

    /// Extract event ID
    pub fn get_event_id(&self) -> Option<String> {
        self.data.event.as_ref().and_then(|e| e.event_id.clone())
    }

    /// Extract issue ID
    pub fn get_issue_id(&self) -> Option<String> {
        self.data.event.as_ref().and_then(|e| e.issue_id.clone())
    }

    /// Extract platform
    pub fn get_platform(&self) -> Option<String> {
        self.data.event.as_ref().and_then(|e| e.platform.clone())
    }

    /// Extract datetime
    pub fn get_datetime(&self) -> Option<String> {
        self.data.event.as_ref().and_then(|e| e.datetime.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_sentry_payload_with_integer_ids() {
        let json = r#"{
            "action": "triggered",
            "data": {
                "event": {
                    "event_id": 4505865345040384,
                    "issue_id": 123456789,
                    "project": 987654321
                }
            },
            "actor": {
                "id": 55555,
                "type": "user"
            }
        }"#;

        let payload: SentryWebhookPayload = serde_json::from_str(json).unwrap();

        assert_eq!(
            payload.data.event.as_ref().unwrap().event_id,
            Some("4505865345040384".to_string())
        );
        assert_eq!(
            payload.data.event.as_ref().unwrap().issue_id,
            Some("123456789".to_string())
        );
        assert_eq!(
            payload.data.event.as_ref().unwrap().project,
            Some("987654321".to_string())
        );
        assert_eq!(
            payload.actor.as_ref().unwrap().id,
            Some("55555".to_string())
        );
    }

    #[test]
    fn test_deserialize_sentry_payload_with_string_ids() {
        let json = r#"{
            "action": "triggered",
            "data": {
                "event": {
                    "event_id": "abc-123",
                    "issue_id": "issue-456",
                    "project": "my-project"
                }
            }
        }"#;

        let payload: SentryWebhookPayload = serde_json::from_str(json).unwrap();

        assert_eq!(
            payload.data.event.as_ref().unwrap().event_id,
            Some("abc-123".to_string())
        );
        assert_eq!(
            payload.data.event.as_ref().unwrap().issue_id,
            Some("issue-456".to_string())
        );
        assert_eq!(
            payload.data.event.as_ref().unwrap().project,
            Some("my-project".to_string())
        );
    }

    #[test]
    fn test_deserialize_sentry_payload_with_float_timestamp() {
        let json = r#"{
            "action": "triggered",
            "data": {
                "event": {
                    "event_id": "test-event",
                    "timestamp": 1768481085.267,
                    "datetime": "2026-01-15T12:44:45.267000Z",
                    "platform": "javascript",
                    "level": "error"
                }
            }
        }"#;

        let payload: SentryWebhookPayload = serde_json::from_str(json).unwrap();
        let event = payload.data.event.as_ref().unwrap();

        assert_eq!(event.timestamp, Some("1768481085.267".to_string()));
        assert_eq!(
            event.datetime,
            Some("2026-01-15T12:44:45.267000Z".to_string())
        );
        assert_eq!(event.platform, Some("javascript".to_string()));
        assert_eq!(event.level, Some("error".to_string()));
    }
}
