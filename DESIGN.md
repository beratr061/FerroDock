---
name: FerroDock
description: Lightweight self-hosted PaaS control plane, styled after Linear's design system
colors:
  canvas-dark: "#010102"
  surface-1-dark: "#0f1011"
  surface-2-dark: "#141516"
  surface-3-dark: "#18191a"
  hairline-dark: "#23252a"
  input-border-dark: "#62666d"
  ink-dark: "#f7f8f8"
  ink-muted-dark: "#d0d6e0"
  ink-subtle-dark: "#8a8f98"
  canvas-light: "#f5f6f6"
  surface-1-light: "#ffffff"
  surface-2-light: "#f6f7f7"
  surface-3-light: "#eeeff1"
  hairline-light: "#e4e5e9"
  input-border-light: "#7c818a"
  ink-light: "#0f1011"
  ink-muted-light: "#3c4149"
  ink-subtle-light: "#62666d"
  primary: "#5e6ad2"
  primary-hover: "#5561cc"
  primary-focus: "#5e69d1"
  on-primary: "#ffffff"
  success-dark: "#27a644"
typography:
  family: Inter (substitute for Linear Text/Display), JetBrains Mono (substitute for Linear Mono)
  body-sm: 14px / 400 / 1.5 (default UI text)
  caption: 12px / 400 / 1.4
  mono: 13px / 400 / 1.5
  body: 16px / 400 / 1.5 / -0.05px
  body-lg: 18px / 1.5 / -0.1px (section headings at 500)
  card-title: 22px / 500 / 1.25 / -0.4px (page titles)
  headline: 28px / 600 / 1.2 / -0.6px
rounded:
  control: 8px
  panel: 12px
  pill: 9999px
---

# FerroDock design

The interface follows the Linear design analysis (`DESIGN-linear.app.md`) as closely as an operations panel allows. Tokens live in `web/src/styles/tokens.css`; components read them through Tailwind theme variables, never hard-coded colours.

## Surfaces

Hierarchy comes from a surface ladder plus 1px hairlines, not shadows.

| Level | Dark | Light | Use |
| --- | --- | --- | --- |
| canvas (`--bg`) | #010102 | #f5f6f6 | page background |
| surface-1 (`--surface`) | #0f1011 | #ffffff | panels, cards, inputs, sidebar |
| surface-2 (`--surface-raised`) | #141516 | #f6f7f7 | selected tab or segment, hovered rows and buttons |
| surface-3 (`--surface-hover`) | #18191a | #eeeff1 | active navigation item |
| hairline (`--border`) | #23252a | #e4e5e9 | panel borders, dividers |

Shadows appear only on floating layers (palette, menus).

## Accent

Lavender `#5e6ad2` is the only chromatic accent: primary buttons, focus rings, the brand mark and link emphasis. It is never a section background or card fill. Status colours (success, warning, danger, info) are the only other hues, and they never fill a surface: they appear as text, icons, a small status dot or a 1px border. Every fill is a solid surface token; translucent fills (alpha modifiers, `opacity-*`, `rgba`, `color-mix`, tints) are blocked by `web/src/styles/fills.test.ts`.

## Type

Inter stands in for Linear's proprietary families; JetBrains Mono for Linear Mono. UI text is 14px. Headings use weight 500 with Linear's negative tracking (22px page titles at -0.4px). Mono is for ids, logs, terminals and code.

## Components

- **Buttons**: 36px high, 8px 14px padding, 8px radius, 14px/500 label. Primary is lavender with white text; secondary is surface-1 with a hairline; ghost is plain text that lifts to surface-2 on hover; danger is surface-1 with danger text and border.
- **Inputs and selects**: 36px, surface-1, 8px radius, 12px side padding, 14px text (16px on phones so iOS does not zoom). Labels are 13px ink-muted at 500. Focus only changes the field's own border to the lavender focus colour, with no separate ring (WCAG 2.4.7 technique C15); hover lightens the border. Checkboxes, buttons and links keep the 2px focus outline.
- **Tabs and segmented controls**: pills. Unselected is subtle text on the canvas; selected lifts to surface-2.
- **Status badges**: 22px surface-2 pills with ink-muted caption text and a 6px dot in the status colour.
- **Alerts**: surface-2 with a status-coloured border and icon; the message stays in ink.
- **Skeletons**: alternate between surface-2 and surface-3 instead of fading.
- **Panels**: surface-1, 12px radius, hairline border.
- **Navigation**: 32px rows, 8px radius; hover lifts to surface-2, the active item to surface-3.

## Deviations from the Linear file

All of these keep WCAG 2.2 AA, which the contrast test in `web/src/styles/contrast.test.ts` enforces:

- Hover on primary buttons is #5561cc instead of #828fff, because white on #828fff is only 2.9:1.
- Inputs use #62666d (dark) and #7c818a (light) borders, because Linear's hairlines are below the 3:1 needed for form fields.
- Linear's site is dark only. FerroDock keeps light, dark and system themes; the light theme is derived from Linear's inverse surfaces with the same accent and ladder.
- The marketing display sizes (40 to 80px) are not used in the panel; the largest heading is the 22px page title.

## Motion

Unchanged: 120 to 280ms transitions with `cubic-bezier(0.16, 1, 0.3, 1)`, only transform, opacity and clip-path, and reduced-motion support (`web/src/styles/motion.css`).
