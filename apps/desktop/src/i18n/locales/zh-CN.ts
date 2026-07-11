import type { Messages } from "../types";

const zhCN = {
  app: {
    name: "OpenDesk",
  },
  nav: {
    home: "首页",
    agent: "Agent",
    chat: "Chat",
    mail: "Mail",
    knowledge: "Knowledge",
  },
  settings: {
    title: "设置",
    language: "语言",
    languageZh: "简体中文",
    languageEn: "English",
  },
  pages: {
    home: {
      title: "OpenDesk",
      description: "架构脚手架 — 请从侧栏选择功能。",
    },
    agent: {
      subtitle: "Sidecar 连通性垂直切片",
      ping: "Ping sidecar",
      pinging: "Pinging...",
    },
    chat: {
      description: "客户会话工作区 — 开发中。",
    },
    mail: {
      description: "收发邮件处理 — 开发中。",
    },
    knowledge: {
      description: "知识库与检索 — 开发中。",
    },
    placeholder: {
      developing: "该功能正在开发中。",
    },
  },
} as const satisfies Messages;

export default zhCN;
