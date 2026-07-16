# Windows vendored OpenSSL build notes

`license-verifier` links OpenSSL via `openssl` crate `vendored` feature (`openssl-src`).

On Windows this requires:

1. **MSVC only** (`stable-x86_64-pc-windows-msvc` / 对应 `--target *-windows-msvc`) — `pnpm tauri`、`pnpm build:license-verifier`、CI release 均使用。**不要**用 GNU host 编桌面端或 verifier。
2. **Perl** — 本地可用 `tooling/strawberry-perl/`（gitignored）；CI 用 `choco install strawberryperl`。

If Perl is missing locally, download once:

If Perl is missing, download once:

```powershell
# from repo root (script already used by agent once)
$dir = "tooling/strawberry-perl"
mkdir $dir -Force
curl.exe -L -o "$dir/strawberry-perl.zip" `
  "https://github.com/StrawberryPerl/Perl-Dist-Strawberry/releases/download/SP_54221_64bit/strawberry-perl-5.42.2.1-64bit-portable.zip"
tar -xf "$dir/strawberry-perl.zip" -C $dir
```

Then:

```bash
pnpm build:license-verifier
```
