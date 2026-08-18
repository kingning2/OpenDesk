const SITE = 'https://kingning2.github.io/OpenDesk';

const FAQS = [
  {
    q: 'OpenDesk 是什么？',
    a: 'OpenDesk 是一个本地优先的 AI 智能客服桌面应用，面向闲鱼、WhatsApp 等渠道的电商卖家。它把 AI 客服回复建议、多渠道账号管理、订单与发货管理整合在一个桌面上。核心逻辑由 Rust 编排，AI 推理由本地 Python 侧车承担。',
  },
  {
    q: 'OpenDesk 支持哪些电商渠道？',
    a: '目前已规划的渠道包括闲鱼与 WhatsApp，并通过统一的 channel 契约层支持扩展更多渠道。',
  },
  {
    q: '我的数据存在哪里？安全吗？',
    a: '数据保存在本机 SQLite 数据库中，由 Rust 层负责读写。AI 只能通过只读 Query Port 查询数据，不能直接写库或自动发送消息，写库与发送始终由你手动操作。',
  },
  {
    q: 'OpenDesk 需要联网吗？AI 在哪里运行？',
    a: '基础能力可离线使用。AI 推理由本地 Python Sidecar 承担，OCR 使用本地 Tesseract 模型（按需下载），不强制依赖云端服务。',
  },
  {
    q: 'OpenDesk 支持哪些操作系统？',
    a: '基于 Tauri 构建，目标平台为 Windows、macOS 与 Linux。',
  },
  {
    q: '现在处于什么开发阶段？',
    a: '当前处于架构骨架与基础切片阶段：UI Shell、Rust Agent 组装、Python Sidecar Ping 已经就绪，尚未正式发布。可以关注 GitHub 仓库的进展。',
  },
  {
    q: '什么是「契约驱动」？',
    a: 'React、Rust、Python 三端共享同一份 contracts 目录作为唯一真相源。任何跨端变更都先修改契约、再同步生成各端类型，从而保证三端接口严格一致。',
  },
  {
    q: 'OpenDesk 是开源的吗？',
    a: '是。源码托管在 GitHub 仓库，后续通过 GitHub Releases 发布安装包，欢迎 Star、Issue 与贡献。',
  },
];

const jsonLd = {
  '@context': 'https://schema.org',
  '@graph': [
    {
      '@type': 'WebSite',
      '@id': `${SITE}/#website`,
      url: `${SITE}/`,
      name: 'OpenDesk',
      alternateName: 'OpenDesk 智能客服',
      description:
        '本地优先的 AI 智能客服桌面应用，面向闲鱼、WhatsApp 电商卖家，提供 AI 回复建议、自动回复、多渠道账号管理与订单发货管理。',
      inLanguage: 'zh-CN',
      publisher: { '@id': `${SITE}/#organization` },
    },
    {
      '@type': 'Organization',
      '@id': `${SITE}/#organization`,
      name: 'OpenDesk',
      url: `${SITE}/`,
      logo: { '@type': 'ImageObject', url: `${SITE}/assets/logo.svg` },
      sameAs: ['https://github.com/kingning2/OpenDesk'],
    },
    {
      '@type': 'SoftwareApplication',
      '@id': `${SITE}/#software`,
      name: 'OpenDesk',
      applicationCategory: 'BusinessApplication',
      operatingSystem: 'Windows, macOS, Linux',
      softwareVersion: '0.1.0',
      inLanguage: 'zh-CN',
      description:
        '本地优先的 AI 智能客服桌面应用。AI 提供客服场景的回复建议与自动回复，核心能力由 Rust 编排，AI 推理由本地 Python 侧车承担，支持闲鱼、WhatsApp 等多渠道账号与订单发货管理。',
      url: `${SITE}/`,
      codeRepository: 'https://github.com/kingning2/OpenDesk',
      offers: { '@type': 'Offer', price: '0', priceCurrency: 'CNY', description: '开源免费' },
      featureList: [
        '实时监听渠道消息',
        '消息过滤与多渠道通知',
        'AI 智能分析与回复建议',
        '关键词自动回复',
        '客服知识库',
        '闲鱼 / WhatsApp 多账号管理',
        '多平台商品互通',
        '商品素材与批量发布',
        '订单与卡券自动发货',
        '评价与黑名单管理',
        '风控日志',
        '定时任务',
        'OCR 图片识别',
        'MCP 与插件扩展',
        '数据本地存储',
      ],
      author: { '@id': `${SITE}/#organization` },
    },
    {
      '@type': 'FAQPage',
      '@id': `${SITE}/#faq`,
      mainEntity: FAQS.map((f) => ({
        '@type': 'Question',
        name: f.q,
        acceptedAnswer: { '@type': 'Answer', text: f.a },
      })),
    },
  ],
};

export default function JsonLd() {
  return (
    <script
      type="application/ld+json"
      dangerouslySetInnerHTML={{ __html: JSON.stringify(jsonLd) }}
    />
  );
}
