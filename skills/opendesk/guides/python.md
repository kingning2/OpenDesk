# Python Guide

适用范围：`python/**`

## 职责

Python 是 **AI Runtime**，不是业务核心。

| 允许 | 禁止 |
|------|------|
| LLM · RAG · OCR · Embedding | GUI · Tauri · React |
| MCP · Agent · Browser 自动化 | SQLite · 业务状态持久化 |
| Queue · Worker · Provider 适配 | 未评审的对外 HTTP Server |

## 目录结构

```
python/
├── sidecar/           # 进程入口
└── packages/
    ├── contracts/     # codegen 类型
    ├── gateway/       # 请求路由
    ├── queue/         # 任务队列
    ├── worker/        # 执行器
    ├── llm/           # 模型调用
    ├── rag/           # 检索
    ├── agent/         # Agent 编排
    └── provider/      # 外部 API 适配
```

## Sidecar 管理面

仅供 Rust 调用，契约：`contracts/openapi/sidecar.v1.yaml`

| 端点 | 用途 |
|------|------|
| `GET /health` | 健康检查 |
| `GET /stats` | 运行时统计 |
| `GET /tasks/active` | 活跃任务 |
| `GET /metrics` | 指标 |
| `GET /debug/dump` | 调试快照 |

## 流式输出路径

```
Python generator  →  Rust 聚合  →  Tauri Events  →  React
```

禁止 Python 直接向前端推事件。

## 包开发

```bash
# 各包独立 pyproject.toml，Ruff 统一配置于根 pyproject.toml
pnpm lint:python
```

## 相关

- [../architecture/layers.md](../architecture/layers.md)
- [../recipes/add-provider.md](../recipes/add-provider.md)
