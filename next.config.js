/** @type {import('next').NextConfig} */
const nextConfig = {
  reactStrictMode: true,
  output: 'export',
  trailingSlash: true,
  images: {
    unoptimized: true,
  },
  assetPrefix: './',
  turbopack: {},
  webpack: (config) => {
    config.experiments = {
      ...config.experiments,
      asyncWebAssembly: true,
    }
    config.resolve.alias = {
      ...config.resolve.alias,
      '@': require('path').resolve(__dirname, 'src'),
      '@/components': require('path').resolve(__dirname, 'src/components'),
      '@/model': require('path').resolve(__dirname, 'src/model'),
      '@/styles': require('path').resolve(__dirname, 'styles'),
      '@/data': require('path').resolve(__dirname, 'src/data'),
      '@/pages': require('path').resolve(__dirname, 'src/pages'),
      '@/utils': require('path').resolve(__dirname, 'src/components/utils'),
    }
    return config
  },
}

module.exports = nextConfig

