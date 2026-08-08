//! 进程内动作工具：让聊天 LLM 跳转页面 / 打开设置分区。
//!
//! 工具本身只做校验；真正的跳转 / 打开设置由前端收到 `chat:message/tool`
//! 事件后执行（见 `apps/desktop/src/features/chat/use-chat.ts`）。

use async_trait::async_trait;
use chat::{ChatTool, ChatToolCaller};
use opendesk_skills::system::{page_by_id, system_pages, SETTINGS_SECTIONS};
use serde_json::{json, Value};

const NAVIGATE_PAGE: &str = "navigate_page";
const OPEN_SETTINGS: &str = "open_settings";

/// 动作工具调用器。
#[derive(Default)]
pub struct SkillActionCaller;

impl SkillActionCaller {
    /// 空调用器。
    pub fn new() -> Self {
        Self
    }
}

fn page_ids() -> Vec<&'static str> {
    system_pages().iter().map(|page| page.id).collect()
}

fn resolve_navigate_page(args: &Value) -> Result<Value, String> {
    let page_id = args
        .get("page")
        .and_then(Value::as_str)
        .ok_or("navigate_page 需要参数 page")?;
    let page = page_by_id(page_id)
        .ok_or_else(|| format!("未知页面 id：{page_id}；可选：{}", page_ids().join(" / ")))?;
    Ok(json!({ "ok": true, "page": page.id, "path": page.path, "label": page.label }))
}

fn resolve_open_settings(args: &Value) -> Result<Value, String> {
    let section = args
        .get("section")
        .and_then(Value::as_str)
        .ok_or("open_settings 需要参数 section")?;
    if !SETTINGS_SECTIONS.contains(&section) {
        return Err(format!(
            "未知设置分区：{section}；可选：{}",
            SETTINGS_SECTIONS.join(" / ")
        ));
    }
    Ok(json!({ "ok": true, "section": section }))
}

#[async_trait]
impl ChatToolCaller for SkillActionCaller {
    fn list_tools(&self) -> Vec<ChatTool> {
        vec![
            ChatTool {
                name: NAVIGATE_PAGE.into(),
                description: format!(
                    "跳转到 OpenDesk 指定页面。page 取页面 id，可选：{}。\
                     用户要去某个页面操作时调用，前端会自动跳转。",
                    page_ids().join(" / ")
                ),
                parameters: json!({
                    "type": "object",
                    "properties": {
                        "page": {
                            "type": "string",
                            "description": "页面 id，如 mail / crawler / customer"
                        }
                    },
                    "required": ["page"]
                }),
            },
            ChatTool {
                name: OPEN_SETTINGS.into(),
                description: format!(
                    "打开 OpenDesk 设置弹窗并定位到分区。section 可选：{}。\
                     用户要配置某项设置时调用，前端会自动打开设置。",
                    SETTINGS_SECTIONS.join(" / ")
                ),
                parameters: json!({
                    "type": "object",
                    "properties": {
                        "section": {
                            "type": "string",
                            "description": "设置分区 id，如 llm / mailIntegration"
                        }
                    },
                    "required": ["section"]
                }),
            },
        ]
    }

    async fn call_tool(&self, name: &str, args: &Value) -> Result<Value, String> {
        match name {
            NAVIGATE_PAGE => resolve_navigate_page(args),
            OPEN_SETTINGS => resolve_open_settings(args),
            other => Err(format!("unknown tool: {other}")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn navigate_page_returns_path_for_known_page() {
        let result = resolve_navigate_page(&json!({ "page": "mail" })).unwrap();
        assert_eq!(result["ok"], true);
        assert_eq!(result["path"], "/features/mail");
        assert_eq!(result["label"], "Mail");
    }

    #[test]
    fn navigate_page_rejects_unknown_page() {
        assert!(resolve_navigate_page(&json!({ "page": "nope" })).is_err());
    }

    #[test]
    fn navigate_page_requires_page_arg() {
        assert!(resolve_navigate_page(&json!({})).is_err());
    }

    #[test]
    fn open_settings_accepts_known_section() {
        let result = resolve_open_settings(&json!({ "section": "llm" })).unwrap();
        assert_eq!(result["section"], "llm");
    }

    #[test]
    fn open_settings_rejects_unknown_section() {
        assert!(resolve_open_settings(&json!({ "section": "nope" })).is_err());
    }

    #[test]
    fn tool_definitions_are_available() {
        let caller = SkillActionCaller::new();
        let names: Vec<String> = caller
            .list_tools()
            .iter()
            .map(|tool| tool.name.clone())
            .collect();
        assert!(names.contains(&NAVIGATE_PAGE.to_string()));
        assert!(names.contains(&OPEN_SETTINGS.to_string()));
    }
}
