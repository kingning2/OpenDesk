# agent

AI 编排层（**自包含**，只依赖外部 crate）— 各功能文件夹，`lib` 编排。

## Purpose

AI 回复管线：意图检测 → 知识注入 → 提示词组装 → LLM 生成 → 议价控制。大功能按文件夹拆分，
各自负责各自功能；`lib.rs` 负责编排与策略（议价控制、provider 分发）。

## 文件夹结构

| 文件夹 | 职责 |
|---|---|
| `model/` | LLM 模型家族：模型 seam（`LlmProvider` trait + 类型 + 工厂）+ 4 个 provider 模块 |
| `knowledge/` | 知识库：商品信息提取与注入（`ItemKnowledge` / `build_item_context`） |
| `prompt/` | 提示词模板：模板独立文件（`templates/*.txt`）+ 意图字符串索引 + 变量插值 |
| `intent/` | 本地意图检测（price / tech / default / no_reply） |
| `reply/` | 回复引擎（编排入口：`ReplyEngine::generate`） |

## 编排（lib）

`lib.rs` 编排 `intent → knowledge → prompt → model → reply`，并 re-export 各文件夹公开 API
（`agent::ChatMessage`、`agent::PromptBuilder`、`agent::ReplyEngine` 等）。

## 边界

- **属于**：AI / LLM 相关逻辑。
- **不属于**：业务存储 / 渠道协议（`platform`）、应用胶水（`business`）、Tauri（`src-tauri`）。

## Extension points

- 新 LLM provider：`model/` 加模块实现 `LlmProvider`，`model/mod.rs` 工厂加分发分支。
- 新意图：`intent/` 加枚举 + 关键词规则；`prompt/` 加模板 + 意图键。
- 新回复策略：改 `reply/`。

## Known Limitations

- provider 静态编译（无运行时插件注册）；DashScope 仅应用级接口；无流式输出。
