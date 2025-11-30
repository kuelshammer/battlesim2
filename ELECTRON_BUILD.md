# Building Battlesim Desktop App (Electron + Rust/WASM)

This document outlines the build process and toolchain used to generate the Battlesim Electron application.

## Prerequisites

*   **Node.js** (v18+)
*   **Rust** (installed via `rustup`)
*   **wasm-pack**: `cargo install wasm-pack`

## Project Architecture

The project combines three technologies:
1.  **Rust (`simulation-wasm/`)**: Contains the core combat simulation logic. Compiled to WebAssembly.
2.  **Next.js (`src/`)**: The React-based frontend user interface.
3.  **Electron (`electron.js`)**: Wraps the Next.js static export into a native desktop application.

## Build Pipeline

The build process is automated via the `npm run build:electron` script (or `build-electron.sh`).

### 1. Compile Rust to WASM
We use `wasm-pack` to compile the Rust code into a WebAssembly module compatible with the browser.

```bash
# In simulation-wasm/
rustup run stable wasm-pack build --target web --out-dir pkg
```
*   **Crucial Step**: We force the `stable` toolchain to ensure the `wasm32-unknown-unknown` target is available and used, avoiding conflicts with system-installed `rustc` versions (e.g., Homebrew).
*   **Artifacts**: The output (`.wasm`, `.js`) is generated in `simulation-wasm/pkg`.

### 2. Synchronize Assets
The generated WASM files must be served by the Next.js application.

```bash
cp simulation-wasm/pkg/* public/
```
*   This copies the `.wasm` binary and the JavaScript glue code to the `public/` directory, where Next.js serves static assets. This solves issues where the webpack-bundled JS bindings might mismatch the runtime-loaded WASM file.

### 3. Build Next.js Frontend
We build the React application as a static site.

```bash
npm run build
# Runs: next build
```
*   **Config**: `next.config.js` uses `output: 'export'` to generate a static HTML/CSS/JS site in the `out/` directory.
*   **Pathing**: `assetPrefix: './'` ensures relative paths work within the Electron `file://` protocol.

### 4. Package with Electron
Finally, `electron-builder` packages the static site and the main Electron process.

```bash
npx electron-builder --mac
```
*   **Config**: `package.json`'s `"build"` section specifies which files to include.
    *   **Important**: `files` must include `"out/**/*"` to bundle the React app.

## Key Files & Configuration

*   **`package.json`**:
    *   `scripts.build:wasm`: Handles the Rust compilation and asset copying.
    *   `build.files`: Whitelists files for the Electron bundle.
*   **`next.config.js`**: Configures static export and WASM support (via `asyncWebAssembly` experiment).
*   **`simulation-wasm/src/simulation.rs`**: Contains the `run_monte_carlo` and `aggregate_results` logic.
    *   *Note*: Aggregation logic handles edge cases where simulation rounds vary (e.g., Total Party Kill), preventing empty result rendering.

## Troubleshooting

*   **"LinkError: function import requires a callable"**: This indicates a version mismatch between the compiled JS bindings and the `.wasm` file. Ensure the `cp pkg/* public/` step ran successfully.
*   **"wasm32-unknown-unknown target not found"**: Ensure `rustup target add wasm32-unknown-unknown` is run and that the build script uses `rustup run stable`.
*   **White Screen**: Open DevTools (uncomment in `electron.js`) to check for path errors (`file:///`). relative paths in `next.config.js` are critical.