# Windows vendored OpenSSL build notes

`license-verifier` links OpenSSL via `openssl` crate `vendored` feature (`openssl-src`).

On Windows this requires:

1. **MSVC only** (`stable-x86_64-pc-windows-msvc` / `x86_64-pc-windows-msvc`) — set by `pnpm tauri`、`pnpm build:license-verifier`、`tauri:*-locked`。**不要**用 GNU host 编桌面端或 verifier。
2. **Perl** — portable Strawberry Perl under `tooling/strawberry-perl/` (gitignored)

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
