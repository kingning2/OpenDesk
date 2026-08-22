# Python Sidecar（单一项目，无 uv workspace）

DingDa 默认能力在 **Rust**。Python 只在 Rust 生态不够时使用（Playwright / Camoufox），由 Rust 托管生命周期。

```
React → Tauri IPC → Rust → Python Sidecar
```

## 目录

```
python/
├── pyproject.toml       # 唯一 Python 项目（dingda-sidecar）
├── sidecar/             # HTTP 进程入口
├── gateway/             # Cookie 续期 / 扫码 / 滑块 / 浏览器
├── shared/              # 日志等
├── contracts/           # codegen 类型（可选引用）
├── browser_data/        # 本地 profile（gitignore）
└── sidecar.spec         # PyInstaller
```

根仓库通过 path 依赖安装本项目，**不再**使用 `[tool.uv.workspace]` 多包。

## 开发

```bash
uv sync
uv run python -m sidecar.main --port 8787
```

## 调用链（Cookie 续期）

```
Rust → POST /v1/channel/cookie_renew
  → gateway.handle_cookie_renew → gateway.cookie_renew
  → gateway.camoufox + gateway.slider
```
