# YouTube RPA 模板图

OpenCV 同款 NCC 模板匹配用的 PNG，需从 **本机 YouTube 截图** 裁剪后替换仓库内占位图。

| 文件 | 裁剪内容 |
|------|----------|
| `more.png` | 频道简介旁的 `…more` / `更多` 按钮 |
| `view_email.png` | About 弹层中的 `View email address` / `查看电子邮件地址` |
| `captcha.png` | reCAPTCHA 复选框区域（用于检测是否需人工验证） |

## 制作步骤

1. 打开目标频道主页，分辨率与日常使用时一致（建议固定 1920×1080）。
2. 用系统截图工具截取按钮区域，尽量紧贴文字/图标边缘。
3. 覆盖本目录下对应 PNG。
4. 若匹配不准，在 `EnrichConfig.match_threshold` 中微调（默认 `0.72`）。

环境变量（可选）：

- `CRAWLER_ENRICH_CHROME_PATH` — Chrome 可执行文件路径
- `CRAWLER_ENRICH_TEMPLATE_DIR` — 模板目录（覆盖默认 `assets/templates`）

依赖：

- 系统已安装 **Google Chrome**
- 系统 PATH 中有 **tesseract**（邮箱 OCR）

算法实现见 [`README.md`](../../README.md)（`imageproc`，无需系统 OpenCV）。
