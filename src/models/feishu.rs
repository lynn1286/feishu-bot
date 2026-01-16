use serde::{Deserialize, Serialize};

/// Feishu webhook message types (Interactive card only)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "msg_type", rename_all = "lowercase")]
pub enum FeishuMessage {
    Interactive { card: Card },
}

// ============ Interactive Card Message ============

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Card {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub header: Option<CardHeader>,
    pub elements: Vec<CardElement>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CardHeader {
    pub title: CardText,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub template: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CardText {
    pub tag: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "tag", rename_all = "lowercase")]
pub enum CardElement {
    Div {
        #[serde(skip_serializing_if = "Option::is_none")]
        text: Option<CardMarkdown>,
        #[serde(skip_serializing_if = "Option::is_none")]
        fields: Option<Vec<CardField>>,
    },
    Hr {},
    Action {
        actions: Vec<CardAction>,
    },
    Note {
        elements: Vec<CardMarkdown>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CardMarkdown {
    pub tag: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CardField {
    pub is_short: bool,
    pub text: CardMarkdown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "tag", rename_all = "lowercase")]
pub enum CardAction {
    Button {
        text: CardText,
        url: String,
        #[serde(rename = "type")]
        button_type: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        width: Option<String>,
    },
}

// ============ Builder Helpers ============

impl FeishuMessage {
    /// Create an interactive card message
    pub fn card(header: Option<CardHeader>, elements: Vec<CardElement>) -> Self {
        FeishuMessage::Interactive {
            card: Card { header, elements },
        }
    }
}

impl CardHeader {
    pub fn new(title: &str, color: Option<&str>) -> Self {
        CardHeader {
            title: CardText {
                tag: "plain_text".to_string(),
                content: title.to_string(),
            },
            template: color.map(|c| c.to_string()),
        }
    }
}

impl CardMarkdown {
    pub fn new(content: &str) -> Self {
        CardMarkdown {
            tag: "lark_md".to_string(),
            content: content.to_string(),
        }
    }
}

impl CardField {
    pub fn new(content: &str, is_short: bool) -> Self {
        CardField {
            is_short,
            text: CardMarkdown::new(content),
        }
    }
}
