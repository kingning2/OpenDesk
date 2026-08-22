# -*- mode: python ; coding: utf-8 -*-
"""PyInstaller spec — freeze DingDa Python sidecar for Tauri externalBin."""

from pathlib import Path

block_cipher = None
root = Path(SPECPATH)

a = Analysis(
    [str(root / "sidecar" / "main.py")],
    pathex=[str(root)],
    binaries=[],
    datas=[],
    hiddenimports=[
        "channels.core.login.helpers",
        "channels.core.login.session",
        "channels.core.login.qrcode",
        "channels.channel",
        "channels.channel_factory",
        "channels.xianyu.xianyu_channel",
        "channels.ali1688.ali1688_channel",
        "channels.core.browser.base",
        "channels.core.platform_config",
        "channels.core.playwright_common",
        "channels.core.camoufox",
        "channels.core.logging",
        "channels.xianyu.browser",
        "channels.ali1688.browser",
        "channels.xianyu.login.qrcode",
        "channels.ali1688.login.qrcode",
        "channels.xianyu.login.cookie_renew",
        "channels.xianyu.login.slider",
        "sidecar.handlers.qr",
        "sidecar.handlers.cookie_renew",
        "sidecar.main",
        "sidecar.server",
        "sidecar.routes",
    ],
    hookspath=[],
    hooksconfig={},
    runtime_hooks=[],
    excludes=[],
    noarchive=False,
    optimize=0,
)
pyz = PYZ(a.pure, a.zipped_data, cipher=block_cipher)

exe = EXE(
    pyz,
    a.scripts,
    a.binaries,
    a.datas,
    [],
    name="sidecar",
    debug=False,
    bootloader_ignore_signals=False,
    strip=False,
    upx=False,
    upx_exclude=[],
    runtime_tmpdir=None,
    console=True,
    disable_windowed_traceback=False,
    argv_emulation=False,
    target_arch=None,
    codesign_identity=None,
    entitlements_file=None,
)
