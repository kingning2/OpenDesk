/** @type {import('next').NextConfig} */
const basePath = process.env.NEXT_PUBLIC_BASE_PATH || '/OpenDesk';

const nextConfig = {
  // 静态导出：无需服务器，纯静态文件部署到 GitHub Pages
  output: 'export',
  // 项目站点挂在 <user>.github.io/OpenDesk/ 子路径下，需 basePath 前缀
  basePath,
  // GitHub Pages 友好：目录生成 index.html
  trailingSlash: true,
  images: { unoptimized: true },
  // 构建期不跑 ESLint（引擎为浏览器 JS，无需 Node 环境 lint；且未装 eslint-config-next）
  eslint: { ignoreDuringBuilds: true },
};

export default nextConfig;
