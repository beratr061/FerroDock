---
name: FerroDock
description: Lightweight self-hosted PaaS control plane
colors:
  bg-light: "#f7f7f7"
  surface-light: "#fcfcfc"
  surface-raised-light: "#f0f0f0"
  surface-hover-light: "#e9e9e9"
  border-light: "#e0e0e0"
  border-strong-light: "#7a7a7a"
  text-light: "#111111"
  text-muted-light: "#3d3d3d"
  text-subtle-light: "#595959"
  primary-light: "#111111"
  on-primary-light: "#f7f7f7"
  primary-hover-light: "#2e2e2e"
  bg-dark: "#0c0c0c"
  surface-dark: "#141414"
  surface-raised-dark: "#1b1b1b"
  surface-hover-dark: "#222222"
  border-dark: "#262626"
  border-strong-dark: "#6e6e6e"
  text-dark: "#f2f2f2"
  text-muted-dark: "#bdbdbd"
  text-subtle-dark: "#9a9a9a"
  primary-dark: "#f2f2f2"
  on-primary-dark: "#0c0c0c"
  primary-hover-dark: "#d0d0d0"
  success-light: "#11602e"
  success-tint-light: "#e6f4ea"
  warning-light: "#7a4a00"
  warning-tint-light: "#fbf0dc"
  danger-light: "#a51f1f"
  danger-tint-light: "#fbe6e6"
  info-light: "#124c96"
  info-tint-light: "#e5effb"
  success-dark: "#5bd58a"
  success-tint-dark: "#13261a"
  warning-dark: "#f2b84b"
  warning-tint-dark: "#2b2412"
  danger-dark: "#ff8a8a"
  danger-tint-dark: "#2e1616"
  info-dark: "#7dbdff"
  info-tint-dark: "#142335"
typography:
  headline:
    fontFamily: "Poppins, system-ui, sans-serif"
    fontSize: "22px"
    fontWeight: 600
    lineHeight: 1.3
    letterSpacing: "-0.01em"
  title:
    fontFamily: "Poppins, system-ui, sans-serif"
    fontSize: "17.5px"
    fontWeight: 600
    lineHeight: 1.4
    letterSpacing: "-0.01em"
  body:
    fontFamily: "Poppins, system-ui, sans-serif"
    fontSize: "14px"
    fontWeight: 400
    lineHeight: 1.5
  label:
    fontFamily: "Poppins, system-ui, sans-serif"
    fontSize: "14px"
    fontWeight: 500
    lineHeight: 1.5
  meta:
    fontFamily: "Poppins, system-ui, sans-serif"
    fontSize: "12px"
    fontWeight: 400
    lineHeight: 1.4
  mono:
    fontFamily: "JetBrains Mono, ui-monospace, monospace"
    fontSize: "13px"
    fontWeight: 400
    lineHeight: 1.5
    fontFeature: "\"tnum\" 1"
rounded:
  control: "6px"
  panel: "10px"
  round: "9999px"
spacing:
  unit: "3.5px"
  control-x: "12px"
  section-gap: "24px"
components:
  button-primary:
    backgroundColor: "{colors.primary-light}"
    textColor: "{colors.on-primary-light}"
    rounded: "{rounded.control}"
    height: "36px"
    padding: "0 12px"
  button-secondary:
    backgroundColor: "{colors.surface-light}"
    textColor: "{colors.text-light}"
    rounded: "{rounded.control}"
    height: "36px"
  button-ghost:
    textColor: "{colors.text-muted-light}"
    rounded: "{rounded.control}"
    height: "36px"
  button-danger:
    backgroundColor: "{colors.danger-tint-light}"
    textColor: "{colors.danger-light}"
    rounded: "{rounded.control}"
    height: "36px"
  input:
    backgroundColor: "{colors.surface-light}"
    textColor: "{colors.text-light}"
    rounded: "{rounded.control}"
    height: "40px"
    padding: "0 10.5px"
  panel:
    backgroundColor: "{colors.surface-light}"
    rounded: "{rounded.panel}"
  nav-item-active:
    backgroundColor: "{colors.surface-raised-light}"
    textColor: "{colors.text-light}"
    rounded: "{rounded.control}"
    height: "40px"
---

# Design System: FerroDock

## Overview

**Creative North Star: "The Quiet Workbench"**

FerroDock is a clean workbench: every tool hangs where it belongs, nothing is there for show, and attention goes to the work on the bench. The interface is a working surface for deploying and watching services, so it stays neutral, precise and calm, and lets real system state be the only thing that speaks loudly.

Every surface is a solid neutral grey derived from black (dark theme) or white (light theme). Colour appears only when it carries meaning: success, warning, danger and info states. Depth comes from tonal layering and 1px opaque lines rather than shadows. Density is moderate: comfortable for daily operation, never cramped, never airy marketing spacing.

Components feel precise and calm: flat at rest, a small press response on activation, quick state changes, and one authored motion moment per product flow (the deploy traffic switch). Gradients, glass, glows and decorative colour were rejected by the owner and stay out.

**Key Characteristics:**
- Solid, opaque neutral surfaces only (R = G = B); semantic status colours are the sole hues.
- WCAG AA floor for every text, AAA for primary and secondary text.
- Poppins for interface text, JetBrains Mono for machine output; tonal layering instead of shadows; light, dark and system themes with identical hierarchy.

## Colors

A graphite-to-paper neutral ramp with four meaning-bearing status hues and nothing else.

### Primary
- **Ink** (light `{colors.primary-light}`, dark `{colors.primary-dark}`): the primary action fill and its pressed hover; the strongest contrast on the page, used once per region for the main action.

### Neutral
- **Workbench Ground** (`{colors.bg-light}` / `{colors.bg-dark}`): page background behind everything.
- **Surface** (`{colors.surface-light}` / `{colors.surface-dark}`): sidebars, panels, lists, inputs.
- **Raised Surface** (`{colors.surface-raised-light}` / `{colors.surface-raised-dark}`): selected navigation, active segments, avatars, skeletons.
- **Hover Surface** (`{colors.surface-hover-light}` / `{colors.surface-hover-dark}`): hover fill for ghost buttons, rows and nav items.
- **Hairline** (`{colors.border-light}` / `{colors.border-dark}`): decorative dividers and panel outlines that carry no information.
- **Control Edge** (`{colors.border-strong-light}` / `{colors.border-strong-dark}`): borders of inputs, selects and secondary buttons; at least 3:1 against every surface.
- **Text** (`{colors.text-light}` / `{colors.text-dark}`): headings, values, primary copy (AAA).
- **Muted Text** (`{colors.text-muted-light}` / `{colors.text-muted-dark}`): descriptions and secondary copy (AAA).
- **Subtle Text** (`{colors.text-subtle-light}` / `{colors.text-subtle-dark}`): small metadata, hints and placeholders (AA, never lower).

### Status
- **Success, Warning, Danger, Info**: each has a solid text colour and a pre-blended opaque tint for its background, plus an opaque border tone. Status text reaches at least 6:1 on both the surface and its own tint.

### Named Rules
**The Signal Only Rule.** A hue appears only when it reports state. If removing the colour would lose no information, the colour must not be there.

**The Opaque Fill Rule.** Fills never use alpha. A soft background is a pre-blended opaque token, so it never shifts with what sits beneath it.

## Typography

**Body Font:** Poppins (with system-ui, sans-serif), weights 400, 500, 600, self-hosted with Latin Extended for Turkish.
**Label/Mono Font:** JetBrains Mono (with ui-monospace) for logs, terminal, key prefixes, identifiers and tabular numbers.

**Character:** Poppins gives a friendly, geometric clarity that reads well at small sizes; JetBrains Mono marks anything the machine produced. Hierarchy comes from weight and colour, not large size jumps.

### Hierarchy
- **Headline** (600, 22px, 1.3): the single heading of authentication screens.
- **Title** (600, 17.5px, 1.4): page titles such as Projects and Settings.
- **Section** (600, 15.75px, 1.4): section headings inside a page (API keys, Appearance).
- **Body** (400, 14px, 1.5): descriptions and running copy; keep measures near 65ch.
- **Label** (500, 14px): form labels, button text, active navigation.
- **Meta** (400, 12-13px): hints, errors under fields, timestamps.
- **Mono** (400, 12-13px, tabular figures): key prefixes, IDs, logs, metrics.

### Named Rules
**The Machine Voice Rule.** Monospace is reserved for text a machine produced or a person must copy exactly. It is never used as decoration to look technical.

## Layout

The root font size is 14px and the spacing unit is a quarter rem (3.5px). Desktop (1024px and wider) shows a 240px sidebar with labels; tablet (768-1023px) collapses it to a 64px icon rail; mobile replaces it with a fixed bottom navigation and a compact top bar holding the logo and account controls. Page content sits in a centred column up to 1152px wide with 14px mobile gutters, 28px on small screens and 35px on desktop. Settings use a 180px section list beside the content on tablet and up, and a horizontally scrolling tab row on mobile. Lists collapse from multi-column rows to stacked rows below 640px. No layout may cause horizontal page scroll at 375px.

## Elevation & Depth

The system is flat. Depth is conveyed by stepping between Ground, Surface and Raised Surface tones and by 1px Hairline or Control Edge borders. The only shadow is reserved for floating layers such as menus and dialogs.

### Shadow Vocabulary
- **Overlay** (`box-shadow: 0 12px 32px` with a near-black at low strength in light theme, deeper in dark theme): menus, popovers and dialogs only.

### Named Rules
**The Flat Bench Rule.** Nothing at rest casts a shadow. If an element seems to need one, step its surface tone instead.

## Shapes

Two radii define the form language: gently squared controls (6px) for buttons, inputs, selects and nav items, and slightly softer panels (10px) for lists, forms and notices. Avatars are fully round. Corners never mix within a component, and coloured side stripes on cards or alerts are not used.

## Components

### Buttons
Precise and calm: flat fill, medium weight label on one line, a 2% press-in on activation.
- **Shape:** gently squared (6px), 36px tall, 12px horizontal padding, optional leading icon.
- **Primary:** Ink fill with inverse text; hover shifts to the softer ink tone.
- **Secondary:** Surface fill with Control Edge border and Text label; hover to Hover Surface.
- **Ghost:** no fill, Muted Text label; hover fills Hover Surface and darkens text.
- **Danger:** Danger tint fill, danger text, danger border; used for confirm-to-destroy steps.
- **Disabled:** Raised Surface fill and Subtle Text, never reduced opacity; busy state shows the action's progress label.

### Cards / Containers
- **Corner Style:** softer panel radius (10px).
- **Background:** Surface on the Ground.
- **Shadow Strategy:** none (The Flat Bench Rule).
- **Border:** 1px Hairline; list rows divide with the same hairline and the last row drops it.
- **Internal Padding:** 14-16px; cards never nest inside cards.

### Inputs / Fields
- **Style:** Surface fill, 1px Control Edge border, 6px radius, 40px tall; label above, hint or error below.
- **Focus:** border shifts to Text plus a 2px focus outline offset 2px.
- **Error:** border and message turn Danger and keep that colour on hover and focus.

### Navigation
- **Sidebar:** 40px items with 20px icons, Muted Text by default, Hover Surface on hover, Raised Surface with Text and medium weight when active.
- **Bottom navigation (mobile):** three equal items, icon above a 12px label, safe-area aware.

### Inline Notices
Tinted, bordered panels with a status icon and status-coloured text; they settle in with a short 4px rise and fade.

### Empty States
A 44px Surface tile holding a single icon, a section-weight heading, one sentence of guidance and at most one action, left-aligned.

## Do's and Don'ts

### Do:
- **Do** take every colour from tokens; light and dark values ship together.
- **Do** keep primary and secondary text at AAA and everything else at AA or better.
- **Do** animate only transform, opacity and clip-path, 100-350ms with `cubic-bezier(0.16, 1, 0.3, 1)`, and honour reduced motion.
- **Do** show skeletons shaped like the final content instead of spinners, and use Phosphor icons in one weight.

### Don't:
- **Don't** use gradients, glassmorphism, glows, blobs or purple-blue accent colour.
- **Don't** use alpha-transparent fills or pure #000 and #fff.
- **Don't** add eyebrow labels above headings, section numbers or decorative status dots.
- **Don't** nest cards or lay out three identical feature cards.
- **Don't** run infinite decorative animation; the only loop is a building indicator that stops when hidden.
