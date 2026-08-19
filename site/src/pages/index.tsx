import { Player } from "@remotion/player";
import type { NextPage } from "next";
import Head from "next/head";
import React from "react";
import { DingDaPromo } from "../remotion/DingDa/DingDaPromo";
import {
  DURATION_IN_FRAMES,
  VIDEO_FPS,
  VIDEO_HEIGHT,
  VIDEO_WIDTH,
} from "../../types/constants";

const SITE = "https://kingning2.github.io/dingda";

const FEATURE_GROUPS: {
  seal: string;
  title: string;
  items: { name: string; desc: string }[];
}[] = [
  {
    seal: "听",
    title: "消息监听",
    items: [
      { name: "实时监听消息", desc: "闲鱼、WhatsApp 会话消息实时同步，入站、出站与建议一条不漏。" },
      { name: "消息过滤", desc: "按关键词与规则过滤消息，屏蔽骚扰与无效会话。" },
      { name: "消息记录留痕", desc: "完整消息日志可回溯，重要对话随时查证。" },
      { name: "多渠道通知", desc: "新消息通过通知渠道提醒，不错过任何一单。" },
    ],
  },
  {
    seal: "智",
    title: "AI 智能",
    items: [
      { name: "AI 智能分析", desc: "理解聊天上下文与买家意图，给出贴合语境的判断。" },
      { name: "AI 回复建议", desc: "自动生成自然回复，一键采纳即可发出。" },
      { name: "关键词自动回复", desc: "命中预设关键词时自动应答，重复劳动交给规则。" },
      { name: "客服知识库", desc: "沉淀话术、商品信息与常见问题，回复始终有据可依。" },
    ],
  },
  {
    seal: "联",
    title: "多账号 · 多平台",
    items: [
      { name: "多账号管理", desc: "闲鱼、WhatsApp 多账号集中管理，一个桌面切换所有店铺。" },
      { name: "多平台商品互通", desc: "商品与素材一处维护，多账号、多平台发布复用。" },
      { name: "渠道扩展", desc: "统一的 channel 契约层，后续可接入更多电商渠道。" },
      { name: "连接状态监控", desc: "账号连接状态实时可见，异常即时反馈。" },
    ],
  },
  {
    seal: "货",
    title: "商品与发布",
    items: [
      { name: "商品素材", desc: "话术、图片、规格模板沉淀，发布时直接套用。" },
      { name: "商品发布", desc: "一键发布商品，地址、素材、内容一次配齐。" },
      { name: "批量发布", desc: "多商品批量上架，批量任务后台运行、进度可查。" },
      { name: "发布记录", desc: "发布历史与状态留档，失败原因一目了然。" },
    ],
  },
  {
    seal: "单",
    title: "订单与发货",
    items: [
      { name: "订单管理", desc: "订单跟进与状态管理，从下单到归档一条线。" },
      { name: "卡券自动发货", desc: "自有 / 对接卡券按规则自动匹配，交付自动化。" },
      { name: "内容变量填充", desc: "发货内容自动替换订单变量，模板一处维护。" },
      { name: "发货延时控制", desc: "按场景控制发货延时，节奏自己说了算。" },
    ],
  },
  {
    seal: "拓",
    title: "运营与扩展",
    items: [
      { name: "评价与黑名单", desc: "评价管理、黑名单拦截，守住店铺口碑与交易安全。" },
      { name: "风控日志", desc: "异常操作与风险事件留档，问题有据可查。" },
      { name: "定时任务", desc: "定时发布、定时提醒，重复动作交给调度。" },
      { name: "OCR · MCP · 插件", desc: "图片文字识别，MCP 与插件体系让能力持续生长。" },
    ],
  },
];

const FAQS: { q: string; a: string }[] = [
  {
    q: "叮答是什么？",
    a: "叮答（DingDa）是一个本地优先的 AI 智能客服桌面应用，面向闲鱼、WhatsApp 等渠道的电商卖家。它把 AI 客服回复建议、多渠道账号管理、订单与发货管理整合在一个桌面上。核心逻辑由 Rust 编排并默认实现 AI；仅当 Rust 生态不够时才使用 Python 侧车。",
  },
  {
    q: "叮答支持哪些电商渠道？",
    a: "目前已规划的渠道包括闲鱼与 WhatsApp，并通过统一的 channel 契约层支持扩展更多渠道。",
  },
  {
    q: "我的数据存在哪里？安全吗？",
    a: "数据保存在本机 SQLite 数据库中，由 Rust 层负责读写。AI 只能通过只读 Query Port 查询数据，不能直接写库或自动发送消息，写库与发送始终由你手动操作。",
  },
  {
    q: "叮答需要联网吗？AI 在哪里运行？",
    a: "基础能力可离线使用。AI 默认由本地 Rust 编排与调用；仅当 Rust 生态缺少可用实现时，才通过本机 Python Sidecar 承担该能力。OCR 使用本地 Tesseract 模型（按需下载），不强制依赖云端服务。",
  },
  {
    q: "叮答支持哪些操作系统？",
    a: "基于 Tauri 构建，目标平台为 Windows、macOS 与 Linux。",
  },
  {
    q: "现在处于什么开发阶段？",
    a: "当前处于架构骨架与基础切片阶段：UI Shell、Rust Agent 组装、Python Sidecar Ping 已经就绪，尚未正式发布。可以关注 GitHub 仓库的进展。",
  },
  {
    q: "什么是「契约驱动」？",
    a: "跨端共享同一份 contracts 目录作为唯一真相源。任何跨端变更都先修改契约、再同步生成各端类型。Python 只在 Rust 生态不够、必须走 sidecar 时才参与实现。",
  },
  {
    q: "叮答是开源的吗？",
    a: "是。源码托管在 GitHub 仓库，后续通过 GitHub Releases 发布安装包，欢迎 Star、Issue 与贡献。",
  },
];

const jsonLd = {
  "@context": "https://schema.org",
  "@graph": [
    {
      "@type": "WebSite",
      "@id": `${SITE}/#website`,
      url: `${SITE}/`,
      name: "叮答",
      alternateName: "叮答 AI 客服",
      description:
        "本地优先的 AI 智能客服桌面应用，面向闲鱼、WhatsApp 电商卖家，提供 AI 回复建议、自动回复、多渠道账号管理与订单发货管理。",
      inLanguage: "zh-CN",
      publisher: { "@id": `${SITE}/#organization` },
    },
    {
      "@type": "Organization",
      "@id": `${SITE}/#organization`,
      name: "叮答",
      url: `${SITE}/`,
      logo: { "@type": "ImageObject", url: `${SITE}/assets/logo.svg` },
      sameAs: ["https://github.com/kingning2/dingda"],
    },
    {
      "@type": "SoftwareApplication",
      "@id": `${SITE}/#software`,
      name: "叮答",
      applicationCategory: "BusinessApplication",
      operatingSystem: "Windows, macOS, Linux",
      softwareVersion: "0.1.0",
      inLanguage: "zh-CN",
      description:
        "本地优先的 AI 智能客服桌面应用。AI 提供客服场景的回复建议与自动回复，核心能力由 Rust 编排并默认实现；仅 Rust 生态不够时才使用 Python 侧车。支持闲鱼、WhatsApp 等多渠道账号与订单发货管理。",
      url: `${SITE}/`,
      codeRepository: "https://github.com/kingning2/dingda",
      offers: { "@type": "Offer", price: "0", priceCurrency: "CNY", description: "开源免费" },
      featureList: [
        "实时监听渠道消息",
        "消息过滤与多渠道通知",
        "AI 智能分析与回复建议",
        "关键词自动回复",
        "客服知识库",
        "闲鱼 / WhatsApp 多账号管理",
        "多平台商品互通",
        "商品素材与批量发布",
        "订单与卡券自动发货",
        "评价与黑名单管理",
        "风控日志",
        "定时任务",
        "OCR 图片识别",
        "MCP 与插件扩展",
        "数据本地存储",
      ],
      author: { "@id": `${SITE}/#organization` },
    },
    {
      "@type": "FAQPage",
      "@id": `${SITE}/#faq`,
      mainEntity: FAQS.map((f) => ({
        "@type": "Question",
        name: f.q,
        acceptedAnswer: { "@type": "Answer", text: f.a },
      })),
    },
  ],
};

const playerStyle: React.CSSProperties = {
  width: "100%",
  display: "block",
};

const Home: NextPage = () => {
  return (
    <div>
      <Head>
        <title>叮答 DingDa — 本地优先的 AI 智能客服平台 | AI 自动回复 · 多账号 · 订单发货</title>
        <meta
          name="description"
          content="叮答（DingDa）是本地优先的 AI 智能客服桌面应用，为闲鱼、WhatsApp 电商卖家提供 AI 回复建议、自动回复、多渠道账号管理、订单与发货管理。数据全程本地存储，开源免费。"
        />
        <meta
          name="keywords"
          content="AI客服,智能客服,闲鱼客服,WhatsApp客服,自动回复,多账号,电商客服,本地优先,开源,Tauri,Rust,AI Agent,DingDa"
        />
        <meta name="author" content="叮答 DingDa" />
        <meta name="robots" content="index, follow" />
        <meta name="viewport" content="width=device-width, initial-scale=1" />
        <meta name="theme-color" content="#F5EDE0" />
        <link rel="canonical" href={`${SITE}/`} />
        <link rel="icon" href="assets/favicon.svg" type="image/svg+xml" />
        <link rel="apple-touch-icon" href="assets/logo.svg" />
        <link rel="alternate" type="text/markdown" href="llms.txt" title="叮答 (LLM-readable)" />

        <meta property="og:type" content="website" />
        <meta property="og:site_name" content="叮答" />
        <meta property="og:title" content="叮答 DingDa — 本地优先的 AI 智能客服平台" />
        <meta
          property="og:description"
          content="面向闲鱼、WhatsApp 电商卖家的 AI 智能客服工作台：AI 回复建议、自动回复、多账号、订单发货，数据全程本地存储，开源免费。"
        />
        <meta property="og:url" content={`${SITE}/`} />
        <meta property="og:image" content={`${SITE}/assets/og-cover.png`} />
        <meta property="og:image:width" content="1200" />
        <meta property="og:image:height" content="630" />
        <meta property="og:locale" content="zh_CN" />

        <meta name="twitter:card" content="summary_large_image" />
        <meta name="twitter:title" content="叮答 DingDa — 本地优先的 AI 智能客服平台" />
        <meta
          name="twitter:description"
          content="面向闲鱼、WhatsApp 电商卖家的 AI 智能客服工作台：AI 回复建议、自动回复、多账号、订单发货，数据全程本地存储。"
        />
        <meta name="twitter:image" content={`${SITE}/assets/og-cover.png`} />

        <script
          type="application/ld+json"
          dangerouslySetInnerHTML={{ __html: JSON.stringify(jsonLd) }}
        />
      </Head>

      {/* Header */}
      <header className="site-header">
        <div className="shell header-inner">
          <a className="brand" href="#top" aria-label="叮答 首页">
            {/* eslint-disable-next-line @next/next/no-img-element */}
            <img src="assets/logo.svg" alt="" width={30} height={30} />
            <span className="brand-word">叮答</span>
          </a>
          <nav className="header-nav" aria-label="主导航">
            <a href="#features">功能</a>
            <a href="#faq">常见问题</a>
            <a className="header-cta" href="https://github.com/kingning2/dingda/releases">
              下载
            </a>
          </nav>
        </div>
      </header>

      <main id="top">
        {/* Hero */}
        <section className="hero" aria-labelledby="hero-title">
          <div className="shell hero-inner">
            <p className="eyebrow">产品宣传片 · 60 秒</p>
            <h1 className="hero-title" id="hero-title">
              本地优先的 AI 智能客服平台
            </h1>
            <p className="hero-sub">
              闲鱼、WhatsApp 多账号一个桌面管理 · AI 智能分析 · 智能回复 · 多平台商品互通 · 数据本地
            </p>
            <div className="video-wrap">
              <Player
                component={DingDaPromo}
                durationInFrames={DURATION_IN_FRAMES}
                fps={VIDEO_FPS}
                compositionWidth={VIDEO_WIDTH}
                compositionHeight={VIDEO_HEIGHT}
                style={playerStyle}
                controls
                autoPlay
                loop
                initiallyMuted
                acknowledgeRemotionLicense
              />
            </div>
          </div>
        </section>

        {/* Features */}
        <section className="section" id="features" aria-labelledby="features-title">
          <div className="shell">
            <p className="eyebrow">功能全景</p>
            <h2 className="section-title" id="features-title">
              卖家需要的，这里都齐了
            </h2>
            <p className="section-sub">从消息进来，到订单归档，一条链路闭环在你的桌面。</p>
            <div className="feature-groups">
              {FEATURE_GROUPS.map((group) => (
                <article className="feature-group" key={group.title}>
                  <div className="feature-group-head">
                    <span className="feature-seal" aria-hidden="true">
                      {group.seal}
                    </span>
                    <h3 className="feature-group-title">{group.title}</h3>
                  </div>
                  <ul className="feature-list">
                    {group.items.map((item) => (
                      <li key={item.name}>
                        <strong>{item.name}</strong>
                        <span>{item.desc}</span>
                      </li>
                    ))}
                  </ul>
                </article>
              ))}
            </div>
          </div>
        </section>

        {/* FAQ */}
        <section className="section section-tint" id="faq" aria-labelledby="faq-title">
          <div className="shell shell-narrow">
            <p className="eyebrow">FAQ</p>
            <h2 className="section-title" id="faq-title">
              你可能想问
            </h2>
            <div className="faq">
              {FAQS.map((f) => (
                <details key={f.q}>
                  <summary>{f.q}</summary>
                  <div className="faq-a">{f.a}</div>
                </details>
              ))}
            </div>
          </div>
        </section>
      </main>

      {/* Footer */}
      <footer className="site-footer">
        <div className="shell footer-inner">
          <div className="footer-grid">
            <div className="footer-brand">
              {/* eslint-disable-next-line @next/next/no-img-element */}
              <img src="assets/logo.svg" alt="" width={34} height={34} />
              <div>
                <p className="footer-name">叮答 DingDa</p>
                <p className="footer-tagline">本地优先的 AI 智能客服平台。数据归你，AI 为辅。</p>
              </div>
            </div>
            <nav className="footer-col" aria-label="社区">
              <p className="footer-col-title">社区</p>
              <a href="https://github.com/kingning2/dingda">GitHub 源码</a>
              <a href="https://github.com/kingning2/dingda/releases">Releases</a>
              <a href="https://github.com/kingning2/dingda/issues">Issue 反馈</a>
            </nav>
            <nav className="footer-col" aria-label="站点">
              <p className="footer-col-title">站点</p>
              <a href="sitemap.xml">Sitemap</a>
              <a href="llms.txt">llms.txt</a>
              <a href="robots.txt">robots.txt</a>
              <a href="assets/promo.mp4">宣传片 mp4</a>
            </nav>
          </div>
          <div className="footer-bottom">
            <p>© 2026 叮答 DingDa · 本地优先 · 开源 · 由 React + Rust 构建（Python 仅补生态缺口）</p>
          </div>
        </div>
      </footer>
    </div>
  );
};

export default Home;
