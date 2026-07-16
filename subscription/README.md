# Subscription Activation Generator

本目录打包出 **两个 exe**：

| 二进制 | 用途 | 是否随主程序分发 |
| --- | --- | --- |
| `activation-gen` | 离线签发 `.key`（私钥签名） | 否，仅管理员本机使用 |
| `license-verifier` | 完整校验（验签 + 机器码 + 过期） | 是，主程序子进程调用 |

主项目通过调用 `license-verifier.exe` 完成校验，验签逻辑不写在主程序源码里。

## 构建

```bash
cargo build --release
```

产物：

```
target/release/activation-gen.exe
target/release/license-verifier.exe
```

复制校验 exe 到主项目（也可在主项目根目录执行）：

```bash
pnpm run build:license-verifier
```

## 图形界面签发（推荐）

仓库根目录：

```bash
pnpm activation-gen:gui
```

或手动：

```powershell
cd subscription
$env:RUSTUP_TOOLCHAIN = "stable-x86_64-pc-windows-msvc"
cargo run --release --features gui --bin activation-gen-gui
```

窗口可填：设备码、天数（从客户首次激活起算）、产品、私钥路径、输出 `.key` 路径；生成后显示 token 预览。私钥默认 `keys/private_key.pem`（勿提交仓库）。

## activation-gen（命令行，可选）

### 生成本机 machineCode

```bash
cargo run --bin activation-gen -- gen-machine-code
```

### 签发激活 key

两种过期策略（二选一）：

| 参数 | 含义 |
| --- | --- |
| `--days N` | **从用户本机首次激活起算** N 天（推荐） |
| `--exp <时间>` | 绝对截止时间（Unix 秒 / RFC3339 / `YYYY-MM-DD`），与是否已激活无关 |

```powershell
# 推荐：30 天从激活瞬间开始计时
cargo run --bin activation-gen -- generate-activation-code `
  --machine-code <machineCode_from_client> `
  --days 30 `
  --out license.key

# 可选：写死日历截止日期
cargo run --bin activation-gen -- generate-activation-code `
  --machine-code <machineCode_from_client> `
  --exp 2026-12-31T23:59:59Z `
  --out license.key
```

`--days` 签发的 token 会带 `durationSecs`；本机首次校验成功时写入 `%APPDATA%/OpenDesk/license.activated_at.json`，之后过期时间 = `activatedAt + durationSecs`。  
**未激活前，计时器不走。** 旧版只含绝对 `exp`、没有 `durationSecs` 的 key 仍按签发时写死的截止时间校验，需用新工具重新签发。

私钥默认读取 `./keys/private_key.pem`，也可用 `--private-key` 指定。

## license-verifier（主程序调用）

公钥在编译时内嵌（`keys/public_key.pem`），运行时无需传公钥路径。

```powershell
# 本机 machineCode
cargo run --bin license-verifier -- gen-machine-code

# 完整校验（--days 授权必须带 --state-dir）
cargo run --bin license-verifier -- verify --key-file license.key --state-dir %APPDATA%\OpenDesk
cargo run --bin license-verifier -- verify --token <activationToken> --state-dir %APPDATA%\OpenDesk
```

退出码：`0` 通过，`1` 未通过，`2` 运行错误。stdout 为一行 JSON（含 `tokenExpiredAt`、可选 `activatedAt`）。

主程序通过 `LICENSE_VERIFIER_EXE` 或 `src-tauri/binaries/license-verifier-<triple>` 定位该 exe（Windows **仅 MSVC 命名**），并自动传入 `--state-dir`。

## CI / Release

`release-desktop` 在各平台 `tauri build` 前会执行：

```bash
node tooling/scripts/build-license-verifier.mjs --target <triple>
```

Windows runner 需安装 Strawberry Perl（workflow 已 `choco install`），供 openssl-src vendored 使用。  
`pnpm tauri build` 的 `beforeBuildCommand`（`build:bundle`）也会再编一次 verifier，并读取 `TAURI_ENV_TARGET_TRIPLE`。


## Key 文件格式

- Header magic: `SFLK`
- Version: `1`
- Payload: obfuscated activation token bytes

## 签名算法

RSA-PSS + SHA256。

- 时长模式（`--days`）：`signingMessage = "{product}|{version}|{machineCode}|dur:{durationSecs}"`
- 绝对过期（`--exp`）：`signingMessage = "{product}|{version}|{machineCode}|{exp}"`

## 安全

- 私钥通过 `.gitignore` 排除，勿提交仓库。
- `license-verifier` 只内嵌公钥，可随安装包分发。
