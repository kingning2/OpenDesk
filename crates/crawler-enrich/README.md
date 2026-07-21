# crawler-enrich

YouTube 频道邮箱补全：**屏幕模板匹配**（OpenCV 同款 NCC 算法，`imageproc` 纯 Rust）+ **enigo 模拟点击**。

仅由 `opendesk-worker` 链接。

## 流程

```text
启动 Chrome（频道主页）
  → 截屏 + 模板匹配 more.png → enigo 点击
  → 截屏 + 匹配 view_email.png → enigo 点击
  → 若匹配 captcha.png → 等待人工过验证（默认 120s）
  → tesseract OCR 全屏 → extract_email
```

## 运行时依赖

- Google Chrome
- Tesseract CLI（`tesseract` 在 PATH 中）
- 模板图：见 [`assets/templates/README.md`](assets/templates/README.md)

## 环境变量

| 变量 | 说明 |
|------|------|
| `CRAWLER_ENRICH_CHROME_PATH` | Chrome 可执行文件 |
| `CRAWLER_ENRICH_TEMPLATE_DIR` | 模板 PNG 目录 |
| `CRAWLER_ENRICH_DELAY_MS` | 任务间隔（Worker） |

## 注意

- RPA 使用**屏幕坐标**；Chrome 须可见，会短暂占用鼠标。
- reCAPTCHA 需人工完成。
- 模板图须从本机 YouTube 截图裁剪（见 templates README）。

## OpenCV 说明

模板匹配算法与 OpenCV `matchTemplate(..., TM_CCOEFF_NORMED)` 等价，由 `imageproc` 实现，**无需安装系统 OpenCV**。若未来需要完整 OpenCV 生态，可再加 optional feature。
