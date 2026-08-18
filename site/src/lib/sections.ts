// OpenDesk 官网场景配置 —— 供 scroll-world 引擎消费。
// 「素材后补」骨架：still 为手绘 clay diorama SVG 占位；clip 先留空，
// 生成场景视频后填入 public/assets/vid/ 下的相对路径即可（见 README）。

const BASE = process.env.NEXT_PUBLIC_BASE_PATH || '/OpenDesk';

const asset = (p: string) => `${BASE}/assets/${p}`;

export interface SwCta {
  primary?: { label: string; href: string };
  secondary?: { label: string; href: string };
}

export interface SwSection {
  id: string;
  label: string;
  still: string;
  clip?: string;
  clipMobile?: string;
  stillMobile?: string;
  accent: string;
  scroll?: number;
  linger?: number;
  eyebrow?: string;
  title?: string;
  body?: string;
  tags?: string[];
  cta?: SwCta;
}

// 6 个场景 = 卖家的价值链条：桌面 → 渠道 → AI 回复 → 订单发货 → 本地优先 → 开源 CTA
export const SECTIONS: SwSection[] = [
  {
    id: 'desk',
    label: '桌面',
    still: asset('stills/desk.svg'),
    accent: '#C8372A',
    scroll: 1.5,
    linger: 0.42,
    eyebrow: 'OpenDesk',
    title: '卖家的客服世界，都在这张桌面上',
    body: '一个本地优先的 AI 智能客服工作台。闲鱼、WhatsApp 多店铺，回到同一个桌面。',
    tags: ['AI 客服', '本地优先', '开源'],
  },
  {
    id: 'channels',
    label: '渠道',
    still: asset('stills/channels.svg'),
    accent: '#C98A3B',
    scroll: 1.4,
    linger: 0.4,
    eyebrow: '多渠道',
    title: '所有店铺，一个入口',
    body: '闲鱼、WhatsApp 多账号集中管理，客服入口不再散落各处。',
    tags: ['闲鱼', 'WhatsApp', '多账号'],
  },
  {
    id: 'ai',
    label: 'AI 回复',
    still: asset('stills/ai.svg'),
    accent: '#7C9A6D',
    scroll: 1.4,
    linger: 0.4,
    eyebrow: 'AI 智能回复',
    title: 'AI 先替你开口',
    body: '理解聊天上下文，给出自然回复建议；命中规则自动回复，拍板权始终在你。',
    tags: ['回复建议', '自动回复', '知识库'],
  },
  {
    id: 'orders',
    label: '订单发货',
    still: asset('stills/orders.svg'),
    accent: '#C9704C',
    scroll: 1.4,
    linger: 0.4,
    eyebrow: '订单与发货',
    title: '从下单到归档',
    body: '订单跟进、发货登记、评价管理，重复动作交给模板与规则。',
    tags: ['订单', '发货', '评价'],
  },
  {
    id: 'local',
    label: '本地优先',
    still: asset('stills/local.svg'),
    accent: '#6E8CA8',
    scroll: 1.4,
    linger: 0.4,
    eyebrow: '本地优先',
    title: '数据留在你的桌面',
    body: '聊天、账号、订单全部存在本机。AI 只读，写库与发送由你操作。',
    tags: ['隐私', '离线可用', '零等待'],
  },
  {
    id: 'open',
    label: '开源扩展',
    still: asset('stills/open.svg'),
    accent: '#8E6E9C',
    scroll: 1.6,
    linger: 0.5,
    eyebrow: '开源与扩展',
    title: '把 AI 客服放回你的桌面',
    body: '源码开源，能力可扩展。去 GitHub 看看进展，或直接下载体验。',
    tags: [],
    cta: {
      primary: { label: '在 GitHub 查看', href: 'https://github.com/kingning2/OpenDesk' },
      secondary: { label: '下载桌面版', href: 'https://github.com/kingning2/OpenDesk/releases' },
    },
  },
];
