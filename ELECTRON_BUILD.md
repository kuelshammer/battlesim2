# Electron Build Guide for Battlesim

This document explains the build process for the Battlesim Electron application. It is intended to help developers (and AI assistants) understand the specific configuration required to build the Next.js + Rust/WASM + Electron stack, specifically targeting macOS (including M1/Apple Silicon).

## Architecture Overview

The application consists of three main parts:
1.  **Frontend:** Next.js (React) application.
2.  **Simulation Core:** Rust code compiled to WebAssembly (WASM).
3.  **Wrapper:** Electron, which serves the static Next.js export.

## Key Configuration Files

### 1. `next.config.js`
To work within Electron (which serves files via the `file://` protocol), Next.js must be configured for static export with relative paths.

```javascript
const nextConfig = {
  output: 'export',        // Exports to /out folder
  trailingSlash: true,     // Required for proper static routing
  images: {
    unoptimized: true,     // Required because Next.js Image Optimization API doesn't work statically
  },
  assetPrefix: './',       // Ensures CSS/JS load relative to the HTML file
  // ... WASM config
}
```

### 2. `electron.js`
The main process entry point. It handles:
- Creating the BrowserWindow.
- Loading `http://localhost:3000` in development.
- Loading `out/index.html` in production/packaged builds.

### 3. `package.json`
Contains specific build configuration for `electron-builder`.
- **Main Entry:** `"main": "electron.js"`
- **Build Configuration:**
  ```json
  "build": {
    "appId": "com.battlesim.app",
    "files": ["electron.js", "out/**/*", "package.json"],
    "mac": {
      "target": {
        "target": "default",
        "arch": ["arm64", "x64"] // Supports M1 (arm64) and Intel (x64)
      }
    }
  }
  ```

## Build Process

### Standard Build Command
The project includes a helper script `build-electron.sh` which automates the process.

```bash
./build-electron.sh
```

This script performs the following steps:
1.  **Next.js Build:** Runs `npm run build`. This compiles the React app and exports static HTML/CSS/JS to the `out/` directory.
2.  **Electron Packaging:** Runs `npx electron-builder`. This takes the `out/` directory and `electron.js` and packages them into a `.dmg` or `.zip`.

### Manual Build Steps

If you need to run steps individually:

1.  **Build Next.js:**
    ```bash
    npm run build
    ```
2.  **Package Electron:**
    ```bash
    # For macOS (auto-detects arch)
    npx electron-builder --mac
    ```

## Development vs. Production

### Development
When running `npm run electron:dev` or `electron .`:
- The Electron app expects a dev server running on port 3000.
- You usually need two terminals:
  1. `npm run dev` (Starts Next.js server)
  2. `npm run electron:dev` (Starts Electron window)

### Production (The Built App)
The built app uses **Static Files** (`out/index.html`).
- **Critical Constraint:** Absolute paths (e.g., `/images/icon.png`) **WILL FAIL** in the built app because they resolve to the root of the file system (e.g., `C:/images/icon.png`).
- **Fix:** Always use relative paths (e.g., `./images/icon.png`) for assets in code.

## WASM Specifics
The Rust simulation is loaded via `simulation-wasm`.
- In `src/components/simulation/simulation.tsx`, the WASM file import must use a relative path for the Electron build to find it:
  ```typescript
  await module.default('./simulation_wasm_bg.wasm') // Relative path for Electron
  ```

## Output
Artifacts are generated in the `dist/` folder.
- `dist/mac-arm64/`: Contains the M1 compatible build.
- `dist/mac/`: Contains the Intel compatible build.
