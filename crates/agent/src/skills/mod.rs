//! AI Skill 基建：可复用能力的注册与查找。
//!
//! 不放业务实现；具体 Skill 由 Feature 或后续模块注册。

pub mod system;

use std::collections::HashMap;
use std::sync::Arc;

use thiserror::Error;

/// Skill 标识（稳定字符串，如 `mail.generate_html`）。
pub type SkillId = String;

/// Skill 元数据（不含业务逻辑）。
#[derive(Debug, Clone)]
pub struct SkillDescriptor {
    /// 唯一标识。
    pub id: SkillId,
    /// 人类可读名称。
    pub name: String,
    /// 一句话说明。
    pub description: String,
}

/// 可注册到 Agent 的 Skill 基建接口。
pub trait Skill: Send + Sync {
    /// Skill 元数据。
    fn descriptor(&self) -> SkillDescriptor;

    /// Skill 正文（指引文本），供注入系统提示词或按需检索。
    fn content(&self) -> String;
}

/// Skill 注册表错误。
#[derive(Debug, Error)]
pub enum SkillError {
    /// 同一 id 重复注册。
    #[error("skill already registered: {0}")]
    AlreadyRegistered(String),
    /// 未找到 Skill。
    #[error("skill not registered: {0}")]
    NotFound(String),
}

/// Skill 注册表。
#[derive(Default)]
pub struct SkillRegistry {
    skills: HashMap<SkillId, Arc<dyn Skill>>,
}

impl SkillRegistry {
    /// 空注册表。
    pub fn new() -> Self {
        Self::default()
    }

    /// 注册 Skill；同 id 重复则报错。
    pub fn register(&mut self, skill: Arc<dyn Skill>) -> Result<(), SkillError> {
        let id = skill.descriptor().id;
        if self.skills.contains_key(&id) {
            return Err(SkillError::AlreadyRegistered(id));
        }
        self.skills.insert(id, skill);
        Ok(())
    }

    /// 按 id 查找 Skill。
    pub fn get(&self, id: &str) -> Result<Arc<dyn Skill>, SkillError> {
        self.skills
            .get(id)
            .cloned()
            .ok_or_else(|| SkillError::NotFound(id.to_string()))
    }

    /// 列出已注册 Skill 元数据。
    pub fn list(&self) -> Vec<SkillDescriptor> {
        self.skills
            .values()
            .map(|skill| skill.descriptor())
            .collect()
    }

    /// 把所有 Skill 的「名称 + 正文」拼成一块指引文本（供注入系统提示词）。
    pub fn guide_text(&self) -> String {
        let mut out = String::new();
        for skill in self.skills.values() {
            let descriptor = skill.descriptor();
            out.push_str(&format!(
                "## {}（{}）\n{}\n\n",
                descriptor.name,
                descriptor.id,
                skill.content()
            ));
        }
        out
    }
}
