//! 内置系统操作指引 Skill：让内置 AI 了解 OpenDesk 的页面、设置与操作路径。
//!
//! 指引内容与前端 `apps/desktop/src/route/nav-registry.ts`、设置分区对齐；
//! 页面 id 同时供 `navigate_page` 工具校验（见 `crates/app/src/app/chat_skills.rs`）。

use std::sync::Arc;

use crate::skills::{Skill, SkillDescriptor, SkillRegistry};

/// 一个系统页面的描述（AI 指引跳转的最小单元）。
#[derive(Debug, Clone, Copy)]
pub struct SystemPage {
    /// 页面 id（供 `navigate_page` 工具引用）。
    pub id: &'static str,
    /// 路由路径。
    pub path: &'static str,
    /// 侧栏显示名。
    pub label: &'static str,
    /// 一句话说明。
    pub summary: &'static str,
}

/// 全部系统页面（与前端 `nav-registry.ts` 对齐）。
pub fn system_pages() -> &'static [SystemPage] {
    &[
        SystemPage {
            id: "home",
            path: "/",
            label: "首页",
            summary: "应用首页",
        },
        SystemPage {
            id: "agent",
            path: "/features/agent",
            label: "Agent",
            summary: "大模型连通性检查",
        },
        SystemPage {
            id: "crawler",
            path: "/features/crawler",
            label: "采集",
            summary: "YouTube 频道采集与过程日志",
        },
        SystemPage {
            id: "crawler-results",
            path: "/features/crawler-results",
            label: "采集结果",
            summary: "已收录频道列表与筛选",
        },
        SystemPage {
            id: "customer",
            path: "/features/customer",
            label: "客户",
            summary: "商务客户档案与合作关系",
        },
        SystemPage {
            id: "chat",
            path: "/features/chat",
            label: "Chat",
            summary: "与内置 AI 对话（当前页）",
        },
        SystemPage {
            id: "mail",
            path: "/features/mail",
            label: "Mail",
            summary: "收发邮件处理",
        },
        SystemPage {
            id: "workflow",
            path: "/features/workflow",
            label: "工作流",
            summary: "邮件/WhatsApp 客户旅程流程与话术库",
        },
        SystemPage {
            id: "knowledge",
            path: "/features/knowledge",
            label: "Knowledge",
            summary: "知识库与检索",
        },
    ]
}

/// 设置弹窗分区 id（与前端 `SettingsSectionId` 对齐）。
pub const SETTINGS_SECTIONS: &[&str] = &["language", "youtube", "llm", "mailIntegration"];

/// 按页面 id 查找页面。
pub fn page_by_id(id: &str) -> Option<&'static SystemPage> {
    system_pages().iter().find(|page| page.id == id)
}

/// 一个指引 Skill 的最小实现（元数据 + 正文）。
struct GuideSkill {
    descriptor: SkillDescriptor,
    body: String,
}

impl Skill for GuideSkill {
    fn descriptor(&self) -> SkillDescriptor {
        self.descriptor.clone()
    }

    fn content(&self) -> String {
        self.body.clone()
    }
}

fn guide(descriptor: SkillDescriptor, body: impl Into<String>) -> GuideSkill {
    GuideSkill {
        descriptor,
        body: body.into(),
    }
}

/// 页面地图文本（供 `system.overview` 引用）。
fn page_map_text() -> String {
    let mut out = String::new();
    for page in system_pages() {
        out.push_str(&format!(
            "- {}(id: {}) 路径 {}：{}\n",
            page.label, page.id, page.path, page.summary
        ));
    }
    out
}

/// 全部内置系统操作指引。
fn builtin_guides() -> Vec<GuideSkill> {
    let overview = guide(
        SkillDescriptor {
            id: "system.overview".into(),
            name: "系统总览与页面地图".into(),
            description: "OpenDesk 系统结构、页面地图与一键跳转工具说明".into(),
        },
        format!(
            "你正在为 OpenDesk 桌面应用的内置 AI 助手服务。用户会问你如何配置功能、去哪个页面操作，\
             请依据下面的系统知识回答，能直接操作就主动操作。\n\n\
             ## 如何到达一个页面\n\
             左侧竖条导航栏（窄轨）排列所有页面图标，点击即可进入；也可以调用 `navigate_page` 工具直接跳转。\n\n\
             ## 页面地图\n{}\n\
             ## 设置弹窗\n\
             右上角齿轮图标打开设置弹窗，包含分区：language（语言）、youtube（YouTube 采集 API Key）、\
             llm（AI/LLM 配置）、mailIntegration（邮件开信追踪）。可用 `open_settings` 工具直接打开到指定分区。\n\n\
             ## 可用动作工具\n\
             - `navigate_page(page)`：跳到指定页面，page 取上面页面地图里的 id。\n\
             - `open_settings(section)`：打开设置弹窗并定位到分区（language/youtube/llm/mailIntegration）。\n\n\
             回答「怎么配置 / 去哪里操作」时：先说结论，再给出可点击的路径；\
             用户明确要去某页或配置某设置时，主动调用对应工具。",
            page_map_text(),
        ),
    );

    let settings_llm = guide(
        SkillDescriptor {
            id: "system.setting.llm".into(),
            name: "快速配置 AI/LLM".into(),
            description: "在设置弹窗 AI/LLM 分区配置模型与 API Key".into(),
        },
        "快速配置 AI/LLM：\n\
         1. 打开设置弹窗（右上角齿轮）→「AI / LLM」分区；或直接调用 `open_settings(\"llm\")`。\n\
         2. 厂商预设：OpenAI / Anthropic / DeepSeek / 豆包（火山方舟）/ Kimi / Ollama / 自定义。\n\
         3. 选择预设后填写模型 ID 与 API Key（Ollama 可留空 Key，模型需本地已拉取），可用「测试连接」验证。\n\
         4. 两个开关：允许工具调用（AI 查询业务数据、一键跳转）、允许长期记忆。\n\
         5. 保存后即可在 Chat 页使用。",
    );

    let settings_mail = guide(
        SkillDescriptor {
            id: "system.setting.mail".into(),
            name: "邮箱相关设置".into(),
            description: "配置邮件开信追踪与邮件账户".into(),
        },
        "邮箱相关设置：\n\
         1. 开信追踪集成：打开设置弹窗 →「邮件开信追踪」分区（`open_settings(\"mailIntegration\")`），\
         配置邮件阅读回执服务并保存。\n\
         2. 邮件账户与收信：在 Mail 页配置，Mail 页 → 账户管理，添加 IMAP/SMTP 账户与授权码。",
    );

    let settings_youtube = guide(
        SkillDescriptor {
            id: "system.setting.youtube".into(),
            name: "配置 YouTube 采集".into(),
            description: "填写 YouTube Data API Key 以启动采集".into(),
        },
        "快速配置 YouTube 采集：\n\
         1. 打开设置弹窗 →「YouTube 采集」分区（`open_settings(\"youtube\")`）。\n\
         2. 填入 YouTube Data API v3 Key 并保存。\n\
         3. 之后在「采集」页即可输入频道或关键词启动采集任务。",
    );

    let page_mail = guide(
        SkillDescriptor {
            id: "system.page.mail".into(),
            name: "Mail 页操作指引".into(),
            description: "收发邮件、写信、管理账户".into(),
        },
        "Mail 页（路径 /features/mail，侧栏「Mail」图标）：收发邮件处理。\n\
         - 收件箱：查看/阅读邮件；「未匹配」面板处理无法自动归类的邮件。\n\
         - 写信：新建邮件，可套用话术库模板、用 AI 生成 HTML 正文。\n\
         - 账户：管理 IMAP/SMTP 账户与授权码。\n\
         用户说「看邮件 / 写信 / 收邮件 / 管理邮箱账户」时跳转到此页。",
    );

    let page_crawler = guide(
        SkillDescriptor {
            id: "system.page.crawler".into(),
            name: "采集页操作指引".into(),
            description: "启动 YouTube 频道采集与查看日志".into(),
        },
        "采集页（路径 /features/crawler，侧栏「采集」图标）：YouTube 频道采集。\n\
         - 输入频道地址或关键词，配置采集选项后启动任务。\n\
         - 查看采集过程日志与进度，可取消任务。\n\
         用户说「采集 YouTube 频道 / 采集关键词」时跳转到此页。",
    );

    let page_crawler_results = guide(
        SkillDescriptor {
            id: "system.page.crawlerResults".into(),
            name: "采集结果页操作指引".into(),
            description: "查看与筛选已收录频道".into(),
        },
        "采集结果页（路径 /features/crawler-results，侧栏「采集结果」图标）：已收录频道列表与筛选。\n\
         用户说「看采集到的频道 / 筛选频道」时跳转到此页。",
    );

    let page_customer = guide(
        SkillDescriptor {
            id: "system.page.customer".into(),
            name: "客户页操作指引".into(),
            description: "管理商务客户档案".into(),
        },
        "客户页（路径 /features/customer，侧栏「客户」图标）：商务客户档案与合作关系。\n\
         - 新建/编辑客户档案（名称、联系人、合作状态等）。\n\
         - 查看客户列表、搜索筛选。\n\
         用户说「管理客户 / 新建客户 / 查客户档案」时跳转到此页。",
    );

    let page_workflow = guide(
        SkillDescriptor {
            id: "system.page.workflow".into(),
            name: "工作流页操作指引".into(),
            description: "查看邮件/WhatsApp 客户旅程流程与话术库".into(),
        },
        "工作流页（路径 /features/workflow，侧栏「工作流」图标）：邮件/WhatsApp 客户旅程。\n\
         左侧为模板列表（含类型与账号绑定数），右侧可查看阶段流程、路由规则与话术库。\n\
         用户说「工作流 / 客户旅程 / 阶段流程 / 话术库」时跳转到此页。",
    );

    let page_knowledge = guide(
        SkillDescriptor {
            id: "system.page.knowledge".into(),
            name: "知识库页操作指引".into(),
            description: "知识库与检索".into(),
        },
        "知识库页（路径 /features/knowledge，侧栏「Knowledge」图标）：知识库与检索。\n\
         用户说「知识库 / 检索资料」时跳转到此页。",
    );

    let page_agent = guide(
        SkillDescriptor {
            id: "system.page.agent".into(),
            name: "Agent 页操作指引".into(),
            description: "大模型连通性检查".into(),
        },
        "Agent 页（路径 /features/agent，侧栏「Agent」图标）：大模型连通性检查。\n\
         - 显示 LLM 连接状态，可点击 Ping 测试。\n\
         用户说「测试 AI 连接 / 模型通不通」时跳转到此页。",
    );

    let page_chat = guide(
        SkillDescriptor {
            id: "system.page.chat".into(),
            name: "Chat 页操作指引".into(),
            description: "与内置 AI 对话的当前页".into(),
        },
        "Chat 页（路径 /features/chat，侧栏「Chat」图标）：与内置 AI 对话。\n\
         - 多会话侧栏：新建/重命名/删除会话。\n\
         - 记忆开关：是否使用跨会话长期记忆。\n\
         用户此刻就在此页，无需跳转。",
    );

    vec![
        overview,
        settings_llm,
        settings_mail,
        settings_youtube,
        page_mail,
        page_crawler,
        page_crawler_results,
        page_customer,
        page_workflow,
        page_knowledge,
        page_agent,
        page_chat,
    ]
}

/// 注册全部内置系统操作指引 Skill。
pub fn system_registry() -> SkillRegistry {
    let mut registry = SkillRegistry::new();
    for skill in builtin_guides() {
        let _ = registry.register(Arc::new(skill));
    }
    registry
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registry_registers_all_builtin_guides() {
        let registry = system_registry();
        assert!(registry.list().len() >= 9);
    }

    #[test]
    fn all_guides_have_nonempty_content() {
        for skill in builtin_guides() {
            assert!(
                !skill.content().trim().is_empty(),
                "empty content: {}",
                skill.descriptor().id
            );
        }
    }

    #[test]
    fn page_ids_and_paths_are_unique() {
        let pages = system_pages();
        let ids: Vec<&str> = pages.iter().map(|page| page.id).collect();
        let paths: Vec<&str> = pages.iter().map(|page| page.path).collect();
        let mut ids_sorted = ids.clone();
        let mut paths_sorted = paths.clone();
        ids_sorted.sort_unstable();
        paths_sorted.sort_unstable();
        ids_sorted.dedup();
        paths_sorted.dedup();
        assert_eq!(ids_sorted.len(), ids.len(), "duplicate page id");
        assert_eq!(paths_sorted.len(), paths.len(), "duplicate page path");
    }

    #[test]
    fn page_by_id_resolves_known_pages() {
        assert_eq!(
            page_by_id("mail").map(|page| page.path),
            Some("/features/mail")
        );
        assert!(page_by_id("nope").is_none());
    }

    #[test]
    fn settings_sections_match_known_sections() {
        assert!(SETTINGS_SECTIONS.contains(&"llm"));
        assert!(SETTINGS_SECTIONS.contains(&"mailIntegration"));
    }
}
