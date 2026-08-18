const GITHUB = 'https://github.com/kingning2/OpenDesk';

export default function Footer() {
  return (
    <footer className="site-footer">
      <div className="post-inner footer-inner">
        <div className="footer-grid">
          <div className="footer-brand">
            <img src="assets/logo.svg" alt="" width="34" height="34" />
            <div>
              <p className="footer-name">OpenDesk</p>
              <p className="footer-tagline">本地优先的 AI 智能客服平台。数据归你，AI 为辅。</p>
            </div>
          </div>

          <nav className="footer-col" aria-label="产品">
            <p className="footer-col-title">产品</p>
            <a href="#faq">常见问题</a>
            <a href={`${GITHUB}/releases`}>下载</a>
          </nav>

          <nav className="footer-col" aria-label="社区">
            <p className="footer-col-title">社区</p>
            <a href={GITHUB}>GitHub 源码</a>
            <a href={`${GITHUB}/releases`}>Releases</a>
            <a href={`${GITHUB}/issues`}>Issue 反馈</a>
          </nav>

          <nav className="footer-col" aria-label="站点">
            <p className="footer-col-title">站点</p>
            <a href="sitemap.xml">Sitemap</a>
            <a href="llms.txt">llms.txt</a>
            <a href="robots.txt">robots.txt</a>
          </nav>
        </div>
        <div className="footer-bottom">
          <p>© 2026 OpenDesk · 本地优先 · 开源 · 由 React + Rust + Python 构建</p>
        </div>
      </div>
    </footer>
  );
}
