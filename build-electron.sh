#!/bin/bash

echo "🍎 Building Battlesim Native App for M1..."

# Build Next.js for production
echo "📦 Building Next.js app..."
npm run build

# Build Electron app
echo "🔨 Building Electron app..."
npx electron-builder --mac --publish=never

echo "✅ Native app built successfully!"
echo "📁 Find your app in: dist/"
ls -la dist/