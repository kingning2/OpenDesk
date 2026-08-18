import type { Metadata } from 'next';
import './globals.css';

const SITE = 'https://kingning2.github.io/OpenDesk';

export const metadata: Metadata = {
  metadataBase: new URL(`${SITE}/`),
  title: 'OpenDesk — 本地优先的 AI 智能客服平台 | AI 自动回复 · 多账号 · 订单发货',
  description:
    'OpenDesk 是本地优先的 AI 智能客服桌面应用，为闲鱼、WhatsApp 电商卖家提供 AI 回复建议、自动回复、多渠道账号管理、订单与发货管理。数据全程本地存储，开源免费。',
  keywords: [
    'AI客服', '智能客服', '闲鱼客服', 'WhatsApp客服', '自动回复',
    '多账号', '电商客服', '本地优先', '开源', 'Tauri', 'Rust', 'AI Agent', 'OpenDesk',
  ],
  authors: [{ name: 'OpenDesk' }],
  alternates: {
    canonical: '/',
    types: {
      'text/markdown': [{ url: `${SITE}/llms.txt`, title: 'OpenDesk (LLM-readable)' }],
    },
  },
  openGraph: {
    type: 'website',
    locale: 'zh_CN',
    siteName: 'OpenDesk',
    title: 'OpenDesk — 本地优先的 AI 智能客服平台',
    description:
      '面向闲鱼、WhatsApp 电商卖家的 AI 智能客服工作台：AI 回复建议、自动回复、多账号、订单发货，数据全程本地存储，开源免费。',
    url: `${SITE}/`,
    images: [{ url: `${SITE}/assets/og-cover.png`, width: 1200, height: 630, alt: 'OpenDesk' }],
  },
  twitter: {
    card: 'summary_large_image',
    title: 'OpenDesk — 本地优先的 AI 智能客服平台',
    description:
      '面向闲鱼、WhatsApp 电商卖家的 AI 智能客服工作台：AI 回复建议、自动回复、多账号、订单发货，数据全程本地存储。',
    images: [`${SITE}/assets/og-cover.png`],
  },
  robots: { index: true, follow: true },
  icons: {
    icon: `${SITE}/assets/favicon.svg`,
    apple: `${SITE}/assets/logo.svg`,
  },
  other: { 'theme-color': '#F5EDE0' },
};

export default function RootLayout({ children }: { children: React.ReactNode }) {
  return (
    <html lang="zh-CN">
      <body>{children}</body>
    </html>
  );
}
