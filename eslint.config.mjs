// ESLint 9 flat config for Next.js 16
import js from "@eslint/js";
import tseslint from "typescript-eslint";

export default [
  // Global ignores
  {
    ignores: [
      ".next/**",
      "node_modules/**",
      "simulation-wasm/pkg/**",
      "simulation-wasm/target/**",
      "e2e/**",           // E2E files have their own tsconfig
      "next-env.d.ts",    // Auto-generated
      "public/**",        // Public folder
      "out/**",           // Static export build output
      "*.template.js",    // Template files
      "fix_templates.js",
      "debug_template.js",
    ],
  },
  // Base JS rules
  js.configs.recommended,
  // TypeScript files
  ...tseslint.configs.recommended,
  {
    files: ["**/*.ts", "**/*.tsx"],
    languageOptions: {
      parser: tseslint.parser,
      parserOptions: {
        project: "./tsconfig.json",
      },
    },
    rules: {
      "@typescript-eslint/no-explicit-any": "warn",
      "@typescript-eslint/no-unused-vars": "warn",
      "no-console": "warn",
      "no-debugger": "warn",
      "no-extra-boolean-cast": "warn",
      "no-constant-binary-expression": "warn",
      "valid-typeof": "warn",
      "no-useless-escape": "warn",
      "no-redeclare": "warn",
      "@typescript-eslint/no-unsafe-function-type": "warn",
      "@typescript-eslint/ban-ts-comment": "warn",
      "no-empty": "warn",
      "prefer-const": "warn",
      "no-empty-pattern": "warn",
      "@typescript-eslint/no-empty-object-type": "warn",
    },
  },
  // Test files - allow any types for mocks
  {
    files: ["**/*.test.ts", "**/*.test.tsx"],
    rules: {
      "@typescript-eslint/no-explicit-any": "off",
    },
  },
  // CommonJS config files (next.config.js, vitest configs, etc.)
  {
    files: ["next.config.js", "vitest.config.ts", "**/*.config.js", "**/*.config.ts", "simulation-wasm/**/*.js", "scripts/**/*.js"],
    languageOptions: {
      ecmaVersion: 2020,
      sourceType: "commonjs",
      globals: {
        require: "readonly",
        module: "readonly",
        __dirname: "readonly",
        __filename: "readonly",
      },
    },
    rules: {
      "no-console": "off",
      "no-undef": "off",
      "@typescript-eslint/no-require-imports": "off",
      "@typescript-eslint/no-unused-vars": "warn",
      "no-extra-boolean-cast": "warn",
      "no-constant-binary-expression": "warn",
      "valid-typeof": "warn",
      "no-useless-escape": "warn",
      "no-redeclare": "warn",
      "@typescript-eslint/no-unsafe-function-type": "warn",
      "@typescript-eslint/ban-ts-comment": "warn",
      "no-empty": "warn",
      "prefer-const": "warn",
      "no-empty-pattern": "warn",
    },
  },
  // Other JS files (use module)
  {
    files: ["**/*.js", "!**/*.config.js", "!next.config.js", "!scripts/**/*.js", "!simulation-wasm/**/*.js"],
    languageOptions: {
      ecmaVersion: 2020,
      sourceType: "module",
      globals: {
        console: "readonly",
        process: "readonly",
        Buffer: "readonly",
      },
    },
    rules: {
      "no-console": "off",
    },
  },
];
