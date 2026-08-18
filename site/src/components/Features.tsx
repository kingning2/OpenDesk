// 功能全景 —— 把 OpenDesk 的能力完整列出。
// 每类一个卡片：印章字 + 类别 + 功能点列表。

const GROUPS: {
  seal: string;
  title: string;
  items: { name: string; desc: string }[];
}[] = [
  {
    seal: '听',
    title: '消息监听',
    items: [
      { name: '实时监听消息', desc: '闲鱼、WhatsApp 会话消息实时同步，入站、出站与建议一条不漏。' },
      { name: '消息过滤', desc: '按关键词与规则过滤消息，屏蔽骚扰与无效会话。' },
      { name: '消息记录留痕', desc: '完整消息日志可回溯，重要对话随时查证。' },
      { name: '多渠道通知', desc: '新消息通过通知渠道提醒，不错过任何一单。' },
    ],
  },
  {
    seal: '智',
    title: 'AI 智能',
    items: [
      { name: 'AI 智能分析', desc: '理解聊天上下文与买家意图，给出贴合语境的判断。' },
      { name: 'AI 回复建议', desc: '自动生成自然回复，一键采纳即可发出。' },
      { name: '关键词自动回复', desc: '命中预设关键词时自动应答，重复劳动交给规则。' },
      { name: '客服知识库', desc: '沉淀话术、商品信息与常见问题，回复始终有据可依。' },
    ],
  },
  {
    seal: '联',
    title: '多账号 · 多平台',
    items: [
      { name: '多账号管理', desc: '闲鱼、WhatsApp 多账号集中管理，一个桌面切换所有店铺。' },
      { name: '多平台商品互通', desc: '商品与素材一处维护，多账号、多平台发布复用。' },
      { name: '渠道扩展', desc: '统一的 channel 契约层，后续可接入更多电商渠道。' },
      { name: '连接状态监控', desc: '账号连接状态实时可见，异常即时反馈。' },
    ],
  },
  {
    seal: '货',
    title: '商品与发布',
    items: [
      { name: '商品素材', desc: '话术、图片、规格模板沉淀，发布时直接套用。' },
      { name: '商品发布', desc: '一键发布商品，地址、素材、内容一次配齐。' },
      { name: '批量发布', desc: '多商品批量上架，批量任务后台运行、进度可查。' },
      { name: '发布记录', desc: '发布历史与状态留档，失败原因一目了然。' },
    ],
  },
  {
    seal: '单',
    title: '订单与发货',
    items: [
      { name: '订单管理', desc: '订单跟进与状态管理，从下单到归档一条线。' },
      { name: '卡券自动发货', desc: '自有 / 对接卡券按规则自动匹配，交付自动化。' },
      { name: '内容变量填充', desc: '发货内容自动替换订单变量，模板一处维护。' },
      { name: '发货延时控制', desc: '按场景控制发货延时，节奏自己说了算。' },
    ],
  },
  {
    seal: '拓',
    title: '运营与扩展',
    items: [
      { name: '评价与黑名单', desc: '评价管理、黑名单拦截，守住店铺口碑与交易安全。' },
      { name: '风控日志', desc: '异常操作与风险事件留档，问题有据可查。' },
      { name: '定时任务', desc: '定时发布、定时提醒，重复动作交给调度。' },
      { name: 'OCR · MCP · 插件', desc: '图片文字识别，MCP 与插件体系让能力持续生长。' },
    ],
  },
];

export default function Features() {
  return (
    <section className="post" id="features" aria-labelledby="features-title">
      <div className="post-inner post-inner-wide">
        <p className="post-eyebrow">功能全景</p>
        <h2 className="post-title" id="features-title">卖家需要的，这里都齐了</h2>
        <p className="post-sub">从消息进来，到订单归档，一条链路闭环在你的桌面。</p>

        <div className="feature-groups">
          {GROUPS.map((group) => (
            <article className="feature-group" key={group.title}>
              <div className="feature-group-head">
                <span className="feature-seal" aria-hidden="true">{group.seal}</span>
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
  );
}
