/** @type {import('next').NextConfig} */
const basePath = process.env.NEXT_PUBLIC_BASE_PATH || "/dingda";

const nextConfig = {
  reactStrictMode: true,
  // 静态导出：纯静态文件，部署到 GitHub Pages
  output: "export",
  // 项目站点挂在 <user>.github.io/dingda/ 子路径下
  basePath,
  // GitHub Pages 友好：目录生成 index.html
  trailingSlash: true,
  images: { unoptimized: true },
};

module.exports = nextConfig;
