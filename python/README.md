# Python Sidecar（单一项目，无 uv workspace）

DingDa 默认能力在 **Rust**。Python 只在 Rust 生态不够时使用（Playwright / Camoufox），由 Rust 托管生命周期。

```
React → Tauri IPC → Rust → Python Sidecar
```

## 目录

```
python/
├── pyproject.toml
├── sidecar/                 # HTTP 进程入口 + handlers
├── channels/
│   ├── channel.py            # Channel 抽象基类
│   ├── channel_factory.py    # create_channel(channel_type)
│   ├── core/                 # 公用基建
│   ├── xianyu/xianyu_channel.py
│   └── ali1688/ali1688_channel.py
├── contracts/
└── sidecar.spec
```

## 依赖关系

```
sidecar/handlers  →  create_channel(platform)  →  channel.qrcode() / renew_cookies()
平台子类覆写 hook（如 Ali1688Qrcode._probe_logged_in）处理差异
```

## 开发

```bash
uv sync
uv run python -m sidecar.main --port 8787
```
