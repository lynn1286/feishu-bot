use chrono::Local;
use sqlx::SqlitePool;

use crate::db::groups::FeishuGroup;
use crate::db::projects;
use crate::models::feishu::{
    CardAction, CardElement, CardField, CardHeader, CardMarkdown, CardText, FeishuMessage,
};
use crate::models::sentry::SentryWebhookPayload;

/// Template variables that can be used in message templates
pub struct TemplateVars {
    pub project: String,
    pub environment: String,
    pub level: String,
    pub title: String,
    pub message: String,
    pub url: String,
    pub timestamp: String,
    pub triggered_rule: String,
    pub platform: String,
    pub event_id: String,
    pub issue_id: String,
    pub tags: String,
    pub culprit: String,
    pub user: String,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct CardFieldConfig {
    pub key: String,
    pub label: String,
    pub enabled: bool,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct CardConfig {
    pub show_fields: Vec<CardFieldConfig>,
    pub show_tags: bool,
    pub show_exception_detail: bool,
}

impl Default for CardConfig {
    fn default() -> Self {
        Self {
            show_fields: vec![
                CardFieldConfig {
                    key: "project".into(),
                    label: "项目".into(),
                    enabled: true,
                },
                CardFieldConfig {
                    key: "environment".into(),
                    label: "环境".into(),
                    enabled: true,
                },
                CardFieldConfig {
                    key: "level".into(),
                    label: "级别".into(),
                    enabled: true,
                },
                CardFieldConfig {
                    key: "timestamp".into(),
                    label: "时间".into(),
                    enabled: true,
                },
                CardFieldConfig {
                    key: "platform".into(),
                    label: "平台".into(),
                    enabled: true,
                },
                CardFieldConfig {
                    key: "culprit".into(),
                    label: "源码位置".into(),
                    enabled: true,
                },
                CardFieldConfig {
                    key: "user".into(),
                    label: "触发用户".into(),
                    enabled: true,
                },
            ],
            show_tags: true,
            show_exception_detail: true,
        }
    }
}

impl TemplateVars {
    pub async fn from_payload(payload: &SentryWebhookPayload, pool: &SqlitePool) -> Self {
        let project_id = payload
            .get_project()
            .unwrap_or_else(|| "Unknown".to_string());
        let project_display = projects::get_display_name(pool, &project_id).await;

        let tags = payload
            .data
            .event
            .as_ref()
            .map(|e| {
                e.tags
                    .iter()
                    .map(|(k, v)| format!("{}: {}", k, v))
                    .collect::<Vec<_>>()
                    .join(", ")
            })
            .unwrap_or_default();

        let event = payload.data.event.as_ref();

        TemplateVars {
            project: project_display,
            environment: payload
                .get_environment()
                .unwrap_or_else(|| "Unknown".to_string()),
            level: payload.get_level().unwrap_or_else(|| "error".to_string()),
            title: payload
                .get_title()
                .unwrap_or_else(|| "No title".to_string()),
            message: payload
                .get_message()
                .unwrap_or_else(|| "No message".to_string()),
            url: payload.get_web_url().unwrap_or_default(),
            timestamp: payload
                .get_datetime()
                .and_then(|dt| chrono::DateTime::parse_from_rfc3339(&dt).ok())
                .map(|dt| {
                    dt.with_timezone(&Local)
                        .format("%Y-%m-%d %H:%M:%S")
                        .to_string()
                })
                .unwrap_or_else(|| Local::now().format("%Y-%m-%d %H:%M:%S").to_string()),
            triggered_rule: payload
                .get_triggered_rule()
                .unwrap_or_else(|| "N/A".to_string()),
            platform: payload.get_platform().unwrap_or_else(|| "N/A".to_string()),
            event_id: payload.get_event_id().unwrap_or_else(|| "N/A".to_string()),
            issue_id: payload.get_issue_id().unwrap_or_else(|| "N/A".to_string()),
            tags,
            culprit: event
                .and_then(|e| e.culprit.clone())
                .unwrap_or_else(|| "N/A".to_string()),
            user: event
                .and_then(|e| e.user.as_ref().and_then(|u| u.username.clone()))
                .or_else(|| event.and_then(|e| e.user.as_ref().and_then(|u| u.email.clone())))
                .or_else(|| event.and_then(|e| e.user.as_ref().and_then(|u| u.ip_address.clone())))
                .unwrap_or_else(|| "N/A".to_string()),
        }
    }

    pub fn get_value(&self, key: &str) -> String {
        match key {
            "project" => self.project.clone(),
            "environment" => self.environment.clone(),
            "level" => self.level.clone(),
            "platform" => self.platform.clone(),
            "culprit" => self.culprit.clone(),
            "user" => self.user.clone(),
            "timestamp" => self.timestamp.clone(),
            "triggered_rule" => self.triggered_rule.clone(),
            "event_id" => self.event_id.clone(),
            "issue_id" => self.issue_id.clone(),
            _ => "N/A".to_string(),
        }
    }

    /// Replace template variables in a string
    pub fn replace(&self, template: &str) -> String {
        template
            .replace("{{project}}", &self.project)
            .replace("{{environment}}", &self.environment)
            .replace("{{level}}", &self.level)
            .replace("{{title}}", &self.title)
            .replace("{{message}}", &self.message)
            .replace("{{url}}", &self.url)
            .replace("{{timestamp}}", &self.timestamp)
            .replace("{{triggered_rule}}", &self.triggered_rule)
            .replace("{{platform}}", &self.platform)
            .replace("{{event_id}}", &self.event_id)
            .replace("{{issue_id}}", &self.issue_id)
            .replace("{{tags}}", &self.tags)
            .replace("{{culprit}}", &self.culprit)
            .replace("{{user}}", &self.user)
    }
}

/// Build a test message for connection testing
pub fn build_test_message(group: &FeishuGroup) -> FeishuMessage {
    // Create test template variables with simulated Sentry alert data
    let vars = TemplateVars {
        project: "测试项目".to_string(),
        environment: "production".to_string(),
        level: "error".to_string(),
        title: "这是一个测试告警".to_string(),
        message: "飞书机器人连接测试成功！如果收到此消息，说明配置正确。".to_string(),
        url: "https://sentry.io/test/issue".to_string(),
        timestamp: Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        triggered_rule: "测试规则".to_string(),
        platform: "javascript".to_string(),
        event_id: "test-event-123".to_string(),
        issue_id: "test-issue-456".to_string(),
        tags: "browser: chrome, os: macos".to_string(),
        culprit: "?(index)".to_string(),
        user: "ip:127.0.0.1".to_string(),
    };

    build_card_message(group, &vars)
}

/// Build a Feishu card message based on group configuration and Sentry payload
pub async fn build_message(
    group: &FeishuGroup,
    payload: &SentryWebhookPayload,
    pool: &SqlitePool,
) -> FeishuMessage {
    let vars = TemplateVars::from_payload(payload, pool).await;
    build_card_message(group, &vars)
}

fn build_card_message(group: &FeishuGroup, vars: &TemplateVars) -> FeishuMessage {
    let config: CardConfig = group
        .card_config_json
        .as_deref()
        .and_then(|s| serde_json::from_str(s).ok())
        .unwrap_or_default();

    let title_template = group
        .card_title_template
        .as_deref()
        .unwrap_or("🚨 Sentry 告警");
    let title = vars.replace(title_template);

    let color = group.card_color.as_deref().unwrap_or("red");
    let show_button = group.card_show_details_button.unwrap_or(1) == 1;

    let header = CardHeader::new(&title, Some(color));

    let mut elements = vec![];

    // Exception Detail
    if config.show_exception_detail {
        elements.push(CardElement::Div {
            text: Some(CardMarkdown::new(&format!(
                "**{}**\n{}",
                vars.title, vars.message
            ))),
            fields: Some(vec![]), // Added fields: Some(vec![])
        });
        elements.push(CardElement::Hr {});
    }

    // Dynamic Fields
    let mut fields = vec![];
    for field_cfg in config.show_fields {
        if field_cfg.enabled {
            let value = vars.get_value(&field_cfg.key);
            fields.push(CardField::new(
                &format!("**{}**\n{}", field_cfg.label, value),
                true,
            ));
        }
    }

    if !fields.is_empty() {
        elements.push(CardElement::Div {
            text: Some(CardMarkdown::new("")), // Added text: Some(CardMarkdown::new(""))
            fields: Some(fields),
        });
    }

    // Tags
    if config.show_tags && !vars.tags.is_empty() {
        elements.push(CardElement::Div {
            text: Some(CardMarkdown::new(&format!("**Tags**: {}", vars.tags))),
            fields: Some(vec![]), // Added fields: Some(vec![])
        });
    }

    // Add Rule and IDs
    elements.push(CardElement::Div {
        text: Some(CardMarkdown::new(&format!(
            "**触发规则**: {}\n**Issue ID**: {} | **Event ID**: {}",
            vars.triggered_rule, vars.issue_id, vars.event_id
        ))),
        fields: Some(vec![]), // Added fields: Some(vec![])
    });

    // Add action button if enabled and URL is available
    if show_button && !vars.url.is_empty() {
        elements.push(CardElement::Action {
            actions: vec![CardAction::Button {
                text: CardText {
                    tag: "plain_text".to_string(),
                    content: "查看详情".to_string(),
                },
                url: vars.url.clone(),
                button_type: "primary".to_string(),
                width: Some("fill".to_string()),
            }],
        });
    }

    FeishuMessage::card(Some(header), elements)
}
