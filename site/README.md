# OpenDesk 官网

OpenDesk 项目官网。**Next.js 静态导出**（`output: 'export'`），基于
[scroll-world](https://github.com/oso95/scroll-world)（MIT）滚动穿越引擎构建，
无后端，产物为纯静态文件，部署到 GitHub Pages。

## 快速开始

```bash
cd site
npm install
npm run dev        # 本地开发 http://localhost:3000
npm run build      # 静态导出到 site/out
```

## 目录

```
site/
├── public/                     # 静态资源（原样拷贝到 out/ 根）
│   ├── assets/
│   │   ├── stills/             # 6 张 clay diorama 场景占位（SVG，可换）
│   │   ├── vid/                # 【素材后补】场景视频 + 连接片段放这里
│   │   ├── logo.svg / favicon.svg / og-cover.png
│   ├── llms.txt                # 面向生成式 AI 引擎的站点摘要（GEO）
│   └── .nojekyll               # 让 GitHub Pages 跳过 Jekyll
├── src/
│   ├── app/
│   │   ├── layout.tsx          # 元数据 / OG / Twitter Card / 主题
│   │   ├── page.tsx            # ScrollWorld + FAQ + 页脚
│   │   ├── globals.css         # 引擎主题（暖黏土 + 朱砂）
│   │   ├── robots.ts           # 生成 robots.txt
│   │   └── sitemap.ts          # 生成 sitemap.xml
│   ├── components/
│   │   ├── ScrollWorld.tsx     # scroll-world 引擎封装（客户端）
│   │   ├── JsonLd.tsx          # JSON-LD 结构化数据（FAQ 与正文一致）
│   │   ├── Faq.tsx / Footer.tsx
│   └── lib/
│       ├── scroll-world.js     # scroll-world 引擎（原样移植，MIT）
│       ├── scroll-world.d.ts
│       └── sections.ts         # 6 场景配置（文案 / accent / 素材路径）
├── next.config.mjs             # output: export + basePath /OpenDesk
└── package.json                # 独立应用，不纳入根 pnpm workspace
```

## 部署（GitHub Pages）

`.github/workflows/pages.yml` 会在推送 `site/**` 到 `main` 后自动执行
`npm ci && npm run build` 并部署 `site/out`。

1. 仓库 **Settings → Pages**，**Source** 选 **GitHub Actions**。
2. 站点地址：`https://<用户名>.github.io/OpenDesk/`。

> 由于挂载在 `/OpenDesk/` 子路径，`next.config.mjs` 里 `basePath` 必须为 `/OpenDesk`。
> 若要部署到自定义域名根路径，设 `NEXT_PUBLIC_BASE_PATH=/` 后重新构建。

## 素材后补（scroll-world 视频管线）

当前场景为手绘 clay diorama SVG 占位（引擎无视频时自动对 still 做缩放过渡）。
想要真实的"滚动穿越"效果，按 scroll-world 规范生成并替换：

1. 在 `sections.ts` 每个场景里补 `clip` / `clipMobile`，路径指向 `assets/vid/xxx.mp4`。
2. 连接片段在 `ScrollWorld.tsx` 的 `connectors` 数组填入（长度 = 场景数 - 1）；
   连接片段的**首尾帧必须与相邻 dive 的实际渲染帧一致**（seam rule），否则会跳变。
3. 视频编码建议（来自 scroll-world `pipeline.md`）：
   - 桌面端：原生分辨率、`crf ~20`、`-g 8`、`+faststart`、无音轨
   - 移动端（可选）：720p、`-g 4`、`crf 23`
4. 逐条估算费用后再生成本条（Monid ~$2.99/条 1080p 8s dive，或用 Higgsfield / Codex CLI）。
5. 生成后无需改代码，替换文件即可；SVG 占位可留作 `still` 海报。

## 自定义

- **换用户名/域名**：全局搜索替换 `kingning2.github.io/OpenDesk`（`layout.tsx`、
  `robots.ts`、`sitemap.ts`、`JsonLd.tsx`、`next.config.mjs`、`llms.txt`、`Footer.tsx`）。
- **改文案/场景**：文案在 `sections.ts`（场景标题/正文/tag）与 `Faq.tsx`；
  `JsonLd.tsx` 的 FAQ 需与正文保持一致。
- **主题**：`globals.css` 的 `--sw-*` 变量（背景/墨色/accent/字体）。

## SEO / GEO 说明

- **SEO**：`layout.tsx` 完整 meta + canonical + Open Graph + Twitter Card；
  `robots.ts` + `sitemap.ts`；语义化 FAQ 正文；`SoftwareApplication` /
  `FAQPage` / `Organization` / `WebSite` 结构化数据。
- **GEO（生成式引擎优化）**：`public/llms.txt` 提供纯文本站点摘要；
  FAQ 用简洁可直接引用的问答句式；页面正文由 SSR 输出，无框架内容可被完整抓取。
