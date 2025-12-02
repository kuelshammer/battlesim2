# Implementation Guide for Battlesim2

This document outlines the architecture, development setup, and key guidelines for contributing to the Battlesim2 project.

## 1. Project Overview

Battlesim2 is a web application designed for simulating battles. It consists of two main parts:

*   **Frontend (Next.js/React/TypeScript):** Provides the user interface for configuring simulations, displaying results, and interacting with the backend.
*   **Backend/Simulation Core (Rust/WebAssembly):** Handles the core battle simulation logic, compiled to WebAssembly (WASM) for execution directly in the browser, providing high performance.

### Key Components:

*   **`src/` (Frontend):** Contains all Next.js application source code.
    *   `src/components/`: Reusable React components, organized by feature area (e.g., `creatureForm`, `simulation`).
    *   `src/data/`: Static data, action definitions, and monster configurations used by the frontend.
    *   `src/model/`: Frontend-specific data models and context for managing simulation state.
    *   `src/pages/`: Next.js page components (e.g., `index.tsx` for the main application page).
    *   `styles/`: SCSS files for styling the application.
*   **`simulation-wasm/` (Rust/WASM):** Contains the Rust project for the battle simulation core.
    *   `simulation-wasm/src/`: Rust source files implementing the simulation logic, including:
        *   `lib.rs`: The entry point for the WASM module.
        *   `simulation.rs`: Core simulation engine.
        *   `model.rs`: Rust data models for creatures, actions, buffs, etc.
        *   `enums.rs`: Enumerations used throughout the simulation.
        *   `actions.rs`, `cleanup.rs`, `targeting.rs`, `resolution.rs`, `aggregation.rs`, `dice.rs`: Modules handling specific aspects of the simulation logic.
    *   `simulation-wasm/pkg/`: Output directory for the compiled WebAssembly module and its JavaScript bindings.
    *   `simulation-wasm/examples/`: Example Rust binaries for debugging specific simulation scenarios (e.g., `debug_monster_r1.rs`, `test_buff_cleanup.rs`).

### Integration:

The Rust WASM module (`simulation-wasm/pkg/`) is integrated into the Next.js frontend. The frontend imports and utilizes the WASM-compiled simulation functions, allowing complex calculations to run efficiently in the browser.

## 2. Development Setup

### Prerequisites:

*   **Node.js:** Recommended version specified in `package.json` (or use `nvm` to manage versions).
*   **Rust:** Install using `rustup` (https://rustup.rs/).
*   **`wasm-pack`:** Tool for building Rust-generated WebAssembly. Install with `cargo install wasm-pack`.
*   **`uv`:** Python package manager used for environment control. Ensure it's installed as per project `README` or instructions.

### Getting Started:

1.  **Clone the repository:**
    ```bash
    git clone [repository-url]
    cd Battlesim2
    ```
2.  **Install Frontend Dependencies:**
    ```bash
    npm install
    ```
3.  **Build the Rust WASM Module:**
    Navigate to the `simulation-wasm` directory and build the WASM package.
    ```bash
    cd simulation-wasm
    wasm-pack build --target web
    # or wasm-pack build --target web --dev for development builds
    cd ..
    ```
    This will generate the `pkg` directory within `simulation-wasm`, containing the WASM module and JS bindings.
4.  **Run the Frontend Development Server:**
    ```bash
    npm run dev
    ```
    The application should now be accessible in your browser, typically at `http://localhost:3000`.

## 3. Building and Testing

### Frontend:

*   **Run Development Server:** `npm run dev`
*   **Build for Production:** `npm run build`
*   **Linting:** `npm run lint` (or relevant lint command)
*   **Testing:** Check `package.json` for specific test commands (e.g., `npm test`).

### Rust WASM:

*   **Build (Development):** `cd simulation-wasm && wasm-pack build --target web --dev && cd ..`
*   **Build (Release):** `cd simulation-wasm && wasm-pack build --target web && cd ..`
*   **Run Tests:** `cd simulation-wasm && cargo test && cd ..`
*   **Run Examples:** `cd simulation-wasm && cargo run --example <example_name> && cd ..` (e.g., `cargo run --example debug_monster_r1`).

## 4. Adding New Features

### Frontend (Next.js/React):

1.  **Identify Component Location:** Determine the most appropriate `src/components/` subdirectory for your new feature's components, or create a new one if necessary.
2.  **Create Components:** Develop your React components, adhering to existing styling conventions (SCSS modules).
3.  **Update Data/Models:** If your feature requires new data structures or static data, update `src/data/` or `src/model/`.
4.  **Integrate with Pages:** Integrate your new components into the relevant Next.js pages in `src/pages/`.
5.  **State Management:** Utilize `src/model/simulationContext.ts` or local component state for managing data.

### Backend (Rust/WASM):

1.  **Identify Module Location:** Determine the most logical `simulation-wasm/src/` module for your new logic. Create a new module if the feature is distinct.
2.  **Implement Logic:** Write your Rust code, ensuring it integrates with existing `model.rs`, `enums.rs`, etc.
3.  **Expose to WASM:** If your new Rust functionality needs to be called from JavaScript, ensure it's exposed in `simulation-wasm/src/lib.rs` using `#[wasm_bindgen]`.
4.  **Write Tests:** Add unit tests for your Rust logic in the appropriate `_test.rs` file or a new one.
5.  **Rebuild WASM:** After making Rust changes, always rebuild the WASM module using `wasm-pack build --target web` (or `--dev`) to reflect changes in the frontend.

## 5. Code Style and Conventions

*   **Frontend:**
    *   Follow standard React/TypeScript best practices.
    *   Adhere to existing SCSS naming conventions and structure.
    *   Ensure proper type safety with TypeScript.
*   **Backend:**
    *   Follow Rust idiomatic patterns and best practices.
    *   Run `cargo fmt` to format your Rust code.
    *   Ensure comprehensive test coverage for simulation logic.

## 6. Important Files and Directories

*   `.gitignore`: Specifies intentionally untracked files to ignore.
*   `package.json`: Frontend project metadata and scripts.
*   `next.config.js`: Next.js configuration.
*   `tsconfig.json`: TypeScript configuration.
*   `Cargo.toml`: Rust project metadata and dependencies.
*   `Cargo.lock`: Exact dependency versions for Rust.
*   `public/`: Static assets served directly by Next.js, including compiled WASM artifacts.

This guide should provide a solid foundation for understanding and contributing to the Battlesim2 project.