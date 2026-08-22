# 模块依赖图（Module Dependency Graph）

由 `cargo` path 依赖生成（`find ... Cargo.toml | grep path =`）。节点 = crate，
边 = `A → B` 表示 A 依赖 B。

**`crates/` 内只依赖共享叶子（`common` / `ports`）+ 外部 crate**；`agent` / `platform` / `infra`
为自包含能力包（大功能拆为内部文件夹，lib 编排）；`business` 与 `src-tauri` 位于其上。

## 分层视图

```text
┌───────────────────────────── dingda (apps/desktop/src-tauri) ─────────────────────────────┐
│  应用壳：IPC 命令、状态注册、Builder（依赖全部 crate）                                        │
└──────────────────────────────────────────────────────────────────────────────────────────────┘
        │
   ┌────┴────────┐
   │   business  │  应用胶水（日志/配置/渠道存储/事件桥/计时）
   └────┬────────┘
        │
   ┌────┴──────────────┐   ┌─────────────┐   ┌─────────────┐
   │      agent        │   │  platform   │   │   infra     │   crates/ 能力包（自包含）
   │ AI 编排层          │   │ 渠道平台全栈  │   │ 基础设施     │
   │ (model/knowledge/ │   │(协议+领域+    │   │(总线+sidecar  │
   │  prompt/intent/   │   │ 共享+Provider+│   │ +适配器)     │
   │  reply 文件夹)     │   │  SQLite)     │   └────┬────────┘
   └────┬──────────────┘   └────┬────────┘        │
        │                       │                 │
   ┌────┴────────┐   ┌──────────┴────┐   ┌────────┴──────┐
   │   common    │   │    ports      │   │    macros     │   crates/ 共享叶子
   │ DTO/契约/错误 │   │  Port traits  │   │ 过程宏        │
   └─────────────┘   └───────────────┘   └───────────────┘
```

> `agent` / `platform` / `infra` 均只依赖共享叶子 `common` / `ports` + 外部 crate；
> 内部按功能拆文件夹（如 `agent::model` / `agent::knowledge` / `agent::prompt`）。

## 边列表（crate → 依赖）

```text
agent      → (外部 crate only)
common     → (leaf)
infra      → common, ports
macros     → (leaf)
platform   → common, ports
ports      → common
business   → common, infra, platform
dingda (src-tauri) → infra, business, agent, common, macros, platform, ports
```

## 校验

无环。`agent` / `platform` / `infra` 自包含（只依赖共享叶子）；大功能在内部按文件夹拆分，
由各自 lib 编排；`business` / `src-tauri` 在上层。

重新生成：

```bash
for f in $(find crates business apps/desktop/src-tauri -maxdepth 2 -name Cargo.toml); do
  pkg=$(basename $(dirname "$f")); [ "$pkg" = src-tauri ] && pkg=dingda
  echo "$pkg => $(grep -E 'path = "\.\.?/' "$f" | sed -E 's/.*path = "([^"]+)".*/\1/' | xargs -n1 basename | tr '\n' ' ')"
done
```
