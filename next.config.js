/** @type {import('next').NextConfig} */
const nextConfig = {
  reactStrictMode: true,
  trailingSlash: true,
  images: {
    unoptimized: true,
  },
  webpack: (config) => {
    config.module.rules.push({
      test: /\.wasm$/,
      type: 'asset/resource',
    });
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

