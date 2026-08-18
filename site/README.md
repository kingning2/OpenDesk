# 叮答（DingDa）官网

叮答（DingDa）官网。基于 **Remotion 官方模板**（`create-video --next-pages-dir`）改造：
Next.js 16（pages 目录）+ Remotion 60 秒产品宣传片，`@remotion/player` 内嵌播放。
`output: 'export'` 静态导出，部署到 GitHub Pages。

## 快速开始

```bash
cd site
npm install
npm run dev        # 本地开发 http://localhost:3000
npm run build      # 静态导出到 site/out
```

## 宣传片（Remotion）

首页首屏是一段 **60s / 1920×1080 / 30fps** 的产品宣传片，由 Remotion 合成，
`@remotion/player` 在网页里直接播放（可拖拽预览、可渲染成 mp4 投放）。

```bash
npm run render       # 渲染 mp4 → out/promo.mp4
npm run render:still # 截取首帧 → out/promo.png
npm run remotion     # Remotion Studio 可视化编辑时间轴
```

### 合成结构

```
src/remotion/
├── index.ts            # registerRoot 入口（供 CLI 渲染）
├── Root.tsx            # 注册 Composition（id: DingDaPromo）
└── DingDa/
    ├── DingDaPromo.tsx   # 60s 主合成：6 场景平滑切换
    ├── timing.ts           # 时间轴（各场景起止帧）
    ├── theme.ts            # 色板（暖黏土 + 朱砂）
    ├── helpers.tsx         # 动画工具 + UI 原子组件
    └── scenes/
        ├── Intro.tsx       # 片头：品牌印章 + 主标题
        ├── Accounts.tsx    # 多账号 · 多平台
        ├── Messages.tsx    # 监听消息
        ├── Ai.tsx          # AI 智能分析 / 智能回复
        ├── Orders.tsx      # 订单与发货
        └── Local.tsx       # 本地优先 + CTA
```

改文案/配色直接改对应场景与 `theme.ts`，保存后网页播放器即时生效。

## 目录

```
site/
├── public/                     # 静态资源（原样拷贝到 out/ 根）
│   ├── assets/                 # logo / favicon / og-cover / promo.mp4
│   ├── llms.txt                # 面向生成式 AI 引擎的站点摘要（GEO）
│   ├── robots.txt / sitemap.xml
│   └── .nojekyll               # 让 GitHub Pages 跳过 Jekyll
├── src/
│   ├── pages/
│   │   ├── index.tsx           # 首页：Header + 宣传片 + 功能全景 + FAQ + 页脚
│   │   └── _app.tsx            # 引入全局样式
│   └── remotion/               # 宣传片合成（见上）
├── styles/global.css           # 主题样式（暖黏土 + 朱砂）
├── types/constants.ts          # 合成参数（时长/分辨率/帧率）
├── next.config.js              # output: export + basePath /dingda
└── package.json                # 独立应用，不纳入根 pnpm workspace
```

## 部署（GitHub Pages）

`.github/workflows/pages.yml` 会在推送 `site/**` 到 `main` 后自动执行
`npm ci && npm run build` 并部署 `site/out`（`upload-pages-artifact@v5`
需 `include-hidden-files: true` 以保留 `.nojekyll`）。

1. 仓库 **Settings → Pages**，**Source** 选 **GitHub Actions**。
2. 站点地址：`https://<用户名>.github.io/dingda/`。

> 由于挂载在 `/dingda/` 子路径，`next.config.js` 里 `basePath` 必须为 `/dingda`。
> 若要部署到自定义域名根路径，设 `NEXT_PUBLIC_BASE_PATH=/` 后重新构建。

## 自定义

- **换用户名/域名**：全局搜索替换 `kingning2.github.io/dingda`（`src/pages/index.tsx`、
  `public/llms.txt`、`public/robots.txt`、`public/sitemap.xml`）。
- **改文案/功能**：宣传片场景在 `src/remotion/dingda/scenes/`；功能全景与 FAQ 在
  `src/pages/index.tsx`（JSON-LD 的 FAQ 需与正文保持一致）。
- **主题**：`styles/global.css` 与 `src/remotion/dingda/theme.ts` 同步修改。

## SEO / GEO 说明

- **SEO**：`index.tsx` 完整 meta + canonical + Open Graph + Twitter Card；
  静态 `robots.txt` + `sitemap.xml`；语义化 FAQ；`SoftwareApplication` /
  `FAQPage` / `Organization` / `WebSite` 结构化数据。
- **GEO（生成式引擎优化）**：`public/llms.txt` 提供纯文本站点摘要；功能全景与
  FAQ 为 SSR 纯文本，可被完整抓取。
