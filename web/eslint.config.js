import js from "@eslint/js";
import boundaries from "eslint-plugin-boundaries";
import reactHooks from "eslint-plugin-react-hooks";
import tseslint from "typescript-eslint";

const layerPolicies = [
  { from: { element: { type: "app" } }, allow: { to: { element: { types: ["app", "feature", "shared"] } } } },
  { from: { element: { type: "feature" } }, allow: { to: { element: { types: ["shared"] } } } },
  { from: { element: { type: "shared" } }, allow: { to: { element: { types: ["shared"] } } } },
];

export default tseslint.config(
  { ignores: ["dist", "public", "src/shared/api/schema.d.ts"] },
  js.configs.recommended,
  ...tseslint.configs.strict,
  {
    files: ["src/**/*.{ts,tsx}"],
    plugins: { boundaries, "react-hooks": reactHooks },
    settings: {
      "import/resolver": { typescript: { project: "./tsconfig.json" } },
      "boundaries/elements": [
        { type: "app", pattern: "src/app" },
        { type: "feature", pattern: "src/features/*" },
        { type: "shared", pattern: "src/shared/*" },
      ],
    },
    rules: {
      ...reactHooks.configs.recommended.rules,
      "boundaries/dependencies": ["error", { default: "disallow", policies: layerPolicies }],
      "max-lines": ["error", { max: 250, skipBlankLines: false, skipComments: false }],
      "max-lines-per-function": ["error", { max: 90, skipBlankLines: true }],
      "max-depth": ["error", 3],
      complexity: ["error", 12],
      "no-console": "error",
      "no-restricted-syntax": [
        "error",
        { selector: "JSXAttribute[name.name='style']", message: "Styles belong in CSS files." },
        { selector: "Literal[value=/#[0-9a-fA-F]{3,8}\\b/]", message: "Use color tokens instead of hex values." },
        { selector: "Literal[value=/\\[[^\\]]*#[0-9a-fA-F]{3,8}/]", message: "Arbitrary color utilities are not allowed." },
      ],
    },
  },
  {
    files: ["scripts/**/*.mjs"],
    languageOptions: { globals: { console: "readonly", process: "readonly", URL: "readonly" } },
  },
);
