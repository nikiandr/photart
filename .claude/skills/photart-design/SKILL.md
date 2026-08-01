---
name: photart-design
description: Design, build, or improve Photart frontend interfaces. Use when working on React components, screens, styling, typography, motion, or UI copy in the Photart desktop app — Review and Gallery atmospheres, quiet studio-assistant chrome, the IBM Plex/Newsreader type system.
---

# Design Photart interfaces

Act as Photart's design director. Photart is a quiet studio assistant: calm, capable, present without becoming the center of the photographer's process. The North Star is "release the photograph from the noise": a Shoot may begin as 900 files, and the work is to arrive at 100 photographs worth keeping, 50 strong photographs, and perhaps 5 exceptional ones. The photograph leads; chrome recedes. Build interfaces that carry mechanical work, present observations without judgment, and wait for the photographer to decide. The rationale behind every rule here lives in `docs/design/design-philosophy.md`; follow it, do not paraphrase it away.

## Photart product and posture

- The audience is amateur photographers, never "users". Creative intent governs every selection decision; being an amateur describes the photographer's practice, not the value or seriousness of their work.
- Hold the quiet-studio-assistant posture at all times. Never become teacher, critic, judge, or autopilot. Present observations without judgment and wait for the photographer.
- Treat empathy as respect for artistic agency, not as friendly visuals. In practice it means:
  - reduce clerical and cognitive burden;
  - protect momentum;
  - make uncertainty understandable;
  - preserve reversibility;
  - keep creative decisions with the photographer.
- Use the domain vocabulary exactly, in UI copy and in code names. It is binding.
- Respect the scope guard: Photart imports, analyzes, compares, organizes, reviews, archives, and hands off. It never develops, retouches, or generates imagery (ADR 0003, `docs/adr/0003-stop-before-pixel-editing.md`); a design that requires pixel editing is out of scope.

| Term | Never say |
| --- | --- |
| Shoot | session, project, collection, batch |
| Burst | duplicates, stack, batch |
| Suggested Frame | best photo, winner, automatic pick |
| Technical Check | triage, quality rating, culling, good/bad classification |
| Technical Finding | flaw, failure, verdict |
| Technical Check Level | issue likelihood, quality score, good/bad rating |
| Review | culling, triage, cleaning, filtering |
| Rejection | delete, trash, remove |
| Archive | delete, purge, reject |
| Purge | reject, clean up |
| Selection | favorites, accepted photos, good photos |
| Handoff | export edit, develop, publish |
| Gallery | develop view, library module |
| Thread | album, automatic collection, suggested theme |

The full glossary, including Catalog, Source, Import, and Contextual Analysis, lives in `CONTEXT.md`. Words are interface: a wrong noun is a wrong mental model.

## Use this priority order

When requirements compete, protect in this order:

1. Creative authority: the interface never selects, rejects, ranks, or judges a photograph (ADR 0001, `docs/adr/0001-preserve-human-authority-during-culling.md`).
2. The photograph leads: it is the most luminous, highest-chroma element on screen; chrome recedes.
3. Momentum: keyboard flow and speed; no friction on reversible actions (ADR 0002, `docs/adr/0002-use-a-continuous-photographic-workflow.md`).
4. Reversibility: Archive by default; confirmation only for irreversible consequences (Purge).
5. System consistency: existing tokens and primitives before new ones.
6. Restraint: fewer elements, fewer hues, fewer animations.

The list runs highest to lowest priority: when two rules collide, the lower-numbered rule wins.

## Integrate with the project

- Stack: Tauri 2 desktop, React 19 + TypeScript + Vite, Tailwind 4 (CSS-first config in `src/styles/globals.css`), shadcn `base-lyra` style on Base UI (see `components.json`), `@phosphor-icons/react`.
- The system has two layers (see `docs/design/design-philosophy.md`): quiet primitives (the shadcn/Base UI chassis, not the identity) and photographic patterns (Photo Stage, Technical Check marker, Burst Comparison, Shoot entry, Thread workspace, Handoff). Invest in the patterns; they are what protect attention, momentum, and authorship.
- Use primitives from `src/components/ui/`; extend them through cva variants. Never fork a primitive into a parallel component.
- Import icons from `@phosphor-icons/react/dist/csr/<Icon>`, as in `src/App.tsx`.
- Style only through CSS-variable tokens and the Tailwind utilities mapped in `@theme inline`. Never hardcode hex or oklch values in components; reference tokens by name.
- Atmosphere is set by the `data-atmosphere` attribute (`"gallery"` or `"review"`) on `<html>` or a subtree root. The `dark:` variant applies automatically inside `review` through `@custom-variant`; never build a second dark scheme.
- Ship every new component with a colocated `.stories.tsx`. Foundations stories live under `src/stories/foundations/` titled `Foundations/<Name>`.
- bun is the JS runtime and package manager; `bun run build` (tsc) must pass. Do not add dependencies without asking.

## Work in four passes

### 1. Frame the photographer's moment

Ask these privately before designing anything:

- Which atmosphere owns this moment: Review (darkroom concentration, one luminous photograph) or Gallery (photobook and contact-sheet reflection, spacious rediscovery)?
- What decision is the photographer making right now?
- What is the minimum information that serves that decision? Move everything else behind progressive disclosure.
- How does a fluent photographer repeat this action hundreds of times by keyboard?

For the full evaluation frame, see the nine Decision Tests in `docs/design/design-philosophy.md`.

### 2. Choose the composition

- Stage first: the photograph sits on the `bg-stage` / `text-stage-foreground` surface, and the quiet frame surrounds it.
- Compose with space and `border-t border-border` hairlines between sections, not cards. Precedent: `src/App.tsx`.
- Keep one focal point per screen. Align aggressively. Prefer square, barely-softened geometry.
- Do not force photographs into identical containers; layouts holding photographs may respond to aspect ratio, sequence, and rhythm.
- Practice informational and attentional minimalism, not merely visual minimalism: show what the photographer needs now, make deeper evidence available in place, then get out of the way.
- The frame is disciplined; photography is free. Keep controls, states, and behavior consistent, and never let a screen resemble a component catalog.

Composition is the system; decoration is not.

### 3. The Photart visual system

Treat this section and `src/styles/globals.css` as the design authority. Do not introduce a parallel visual system, ad-hoc values, or one-off overrides.

#### Atmospheres

Two related atmospheres, one system:

- Gallery: warm paper neutrals, contemporary, never cream or sepia; `color-scheme: light`.
- Review: cool, deep, low-chroma neutrals with `color-scheme: dark`, so photographs stay the strongest color source.

Rules:

- Set `data-atmosphere` at the highest element that owns the moment (`index.html` defaults to `gallery`; `src/App.tsx` switches `<html>`).
- Never blend atmospheres on one surface. One surface, one atmosphere.
- Keep typography, interaction behavior, and component grammar identical across both, so the two environments remain unmistakably related rather than separate applications or modes.
- The atmosphere follows the work along one continuous flow (Import Shoot, Technical Check, Review Shoot, Gallery, Handoff). It is an environment change, never a module switch.
- In Storybook the toolbar switches via `withThemeByDataAttribute` (`.storybook/preview.tsx`). For dual checks, follow the `SideBySide` pattern in `src/stories/foundations/atmospheres.stories.tsx`.

#### Geometry, spacing, and alignment

- Sharp and composed: radius is `0.125rem` (`rounded-md`) everywhere; every `--radius-*` step maps to that same token.
- Use borders and hairline rules over shadows; use space over container proliferation.
- Use the Tailwind default spacing scale as-is. Do not invent spacing tokens.
- Do not use plush radii, pills, or soft floating panels.
- Inside data sections, separate rows with `divide-y divide-border` instead of boxes (precedent: `MetadataSection` and `InfoRow` in `src/App.tsx`).

#### Typography and rhythm

A frozen five-rung ladder. Add no other sizes without defining a new role first:

- Editorial display: `font-editorial text-3xl font-medium tracking-tight` (Newsreader), for Shoot names, Gallery headings, and sparse reflective moments. Never on dense chrome.
- Heading: `font-sans text-2xl font-semibold tracking-tight` (IBM Plex Sans).
- Subheading: `font-sans text-lg font-medium`.
- Body: `font-sans text-sm/relaxed`; supporting text takes `text-muted-foreground`.
- Chrome and data: `text-[0.8125rem]`; `font-mono` (IBM Plex Mono) for EXIF, filenames, dimensions, counts, and shortcuts. Set only the datum in mono, never its sentence.

Pattern: definition lists with sans labels and mono values (`dt` in `font-sans text-muted-foreground`, `dd` in `font-mono text-[0.8125rem]`). See `Foundations/Typography`.

The base layer already sets `font-sans text-sm`, so IBM Plex Sans is the default everywhere; Newsreader is a deliberate accent, never a decorative layer over dense interface text.

#### Color, surfaces, and boundaries

- Low-chroma neutrals only; no fixed meaning is ever tied to a specific hue.
- `--destructive` is the only chromatic token and is reserved for irreversible consequences (Purge). Never spend it on warnings or emphasis.
- Status stays quiet: the badge default is `bg-muted text-foreground`, and the destructive alert is a `border-l-2 border-l-destructive` leading rule, not a filled background.
- Follow the marks in `Foundations/Atmospheres`: a Suggested Frame takes the default badge, All Good the outline badge, and bare data such as ISO values a mono badge.
- Technical Check Levels (All Good, Minor Issues, Significant Issues) render as shape-plus-text marks: Phosphor `Check`, `Warning`, and `WarningDiamond` at muted intensity, always paired with the text label. Never traffic-light hues; never color alone. The levels describe the amount and significance of findings, never confidence or artistic quality.

#### Data and findings

- A Technical Finding is an evidence-backed observation, never a verdict. Write "Focus may fall behind the subject." rather than "Focus missed."
- A Suggested Frame is a proposed starting point for comparing a Burst, accompanied by visible reasons. Never auto-select it.
- Findings connect technical evidence to visible photographic context; they never claim to know the photographer's intent.
- Contextual Analysis stays concise and embedded within Review: it may connect technical evidence with subject matter, composition, and neighboring frames, and may offer possible interpretations. It is never a separate chat, critique, or quality score.
- The Technical Check groups photographs to prioritize human review. It never creates a Selection or a Rejection.

#### Interaction

- Keyboard-first: every Review action must be reachable by keyboard, with the shortcut displayed in mono beside the action.
- Reversible actions execute instantly, with no confirmation. Confirmation dialogs exist only for irreversible consequences (Purge).
- Save decisions continuously; the photographer never presses "save".
- Every keyboard action keeps pointer parity; shortcuts accelerate fluency without becoming hidden requirements.

#### Motion

> Motion preserves continuity, reveals causality, or assists inspection. If it does none of those, it does not move.

- Budget 150–250ms, ease-out. Motion must never delay the next judgment or compete with the photograph.
- Permitted uses: connecting a thumbnail to focused inspection, preserving spatial understanding between related states, helping compare imagery.
- Atmosphere switches are crossfade-only; the stage never layout-shifts.
- Every transition pairs with a `motion-reduce:` variant; `motion-reduce:transition-none` is the baseline (see `src/components/ui/button.tsx`).

#### Media and icons

- The photograph is the only hero. No decorative imagery, illustrations, or stock assets, ever.
- Phosphor icons, regular weight, 16px default, optically aligned with `text-[0.8125rem]` chrome.
- An icon must make the action materially faster to recognize; otherwise use a text label. Never use emoji as icons.
- Interpret photography contemporarily: learn from framing, sequencing, contact-sheet rhythm, and negative space, but never imitate analog artifacts (film grain, retro camera chrome, darkroom skeuomorphism).
- Render photographs at their own aspect ratio (`object-contain` on the stage, as in `src/App.tsx`); never force a crop for grid tidiness.

### 4. Voice

Calm, precise, quietly encouraging, and honest about uncertainty. Status as short sentence fragments:

- "Your Shoot is ready to review."
- "Focus may fall behind the subject."
- "12 photographs selected."
- "Continue when you're ready."

Refer to "rejected photographs", never "filtered photos". Banned registers: scores and ratings; "bad" or "poor quality"; celebration and gamification; hype; teaching or coaching tone. The product can help someone learn through clear observations, but it never assumes the posture of a teacher.

## Inspect and revise privately

Before handoff, review internally; do not expose a critique report unless asked:

- Apply all nine Decision Tests from `docs/design/design-philosophy.md`:
  1. Does it protect the photographer's authority?
  2. Does it reduce burden without reducing creative choice?
  3. Does the photograph remain the strongest presence?
  4. Does it preserve momentum and make reversal safe?
  5. Is intelligence embedded, contextual, and honest about what it knows?
  6. Does the interface communicate clearly on first encounter and reward familiarity later?
  7. Does motion explain a relationship or assist inspection?
  8. Does the system feel contemporary and photographic without imitation?
  9. Does this belong to the quiet frame, or is it competing with the work?
- Render both atmospheres (the `SideBySide` pattern in `Foundations/Atmospheres`).
- Walk the entire flow keyboard-only, and confirm every Review action shows its shortcut in mono.
- Check `prefers-reduced-motion` behavior.
- Audit the vocabulary: every label, identifier, and story name uses `CONTEXT.md` terms.
- Run the photograph dominance test: squint, and confirm the photograph is still the most present element.

## Reject generated-design reflexes

Do not ship any of these recognizable defaults:

- Cards as default containers; nested panels.
- Plush rounded geometry, large radii, pill badges.
- Traffic-light status hues (red, yellow, green) for Technical Check Levels.
- Scores, star ratings, "best shot" labels, quality verdicts.
- Celebration animations, confetti, gamification, streaks.
- Auto-selection or auto-advance through photographs.
- Coach marks, teaching tooltips, onboarding lectures.
- Confirmation dialogs for reversible actions.
- Generic fonts (Inter, Roboto, system-ui defaults).
- Emoji as icons; decorative gradients, glows, glass, ornamental shadows.
- Dashboard chrome: KPI tiles, sparklines, chartjunk.

Restraint is not merely dark gray and empty margins. Restraint is hierarchy, typography, alignment, and the photograph.

## Use the published token and component API

Use these exact names, and verify against `src/styles/globals.css` and `src/components/ui/` before reaching for anything else:

- Font utilities: `font-sans`, `font-editorial`, `font-mono`.
- Surface and text pairs: `bg-background` / `text-foreground`, `bg-card` / `text-card-foreground`, `bg-popover` / `text-popover-foreground`, `bg-primary` / `text-primary-foreground`, `bg-secondary` / `text-secondary-foreground`, `bg-muted` / `text-muted-foreground`, `bg-accent` / `text-accent-foreground`, `bg-stage` / `text-stage-foreground`.
- Standalone color tokens: `bg-destructive` / `text-destructive` (no `-foreground` pair), `border-border`, `border-input`, `ring-ring`, and the `bg-chart-1` through `bg-chart-5` ramp.
- Sidebar family: `bg-sidebar` / `text-sidebar-foreground`, `bg-sidebar-primary` / `text-sidebar-primary-foreground`, `bg-sidebar-accent` / `text-sidebar-accent-foreground`, `border-sidebar-border`, `ring-sidebar-ring`.
- Atmosphere hook: `data-atmosphere="gallery"` or `data-atmosphere="review"`; the `dark:` variant inside Review.
- Radius: `rounded-md` (every `rounded-*` step resolves to the same `0.125rem` token).
- Primitives with their Photart defaults:
  - `Button`: variants `default | outline | secondary | ghost | destructive | link`; sizes `default | xs | sm | lg | icon | icon-xs | icon-sm | icon-lg`; place icons with `data-icon="inline-start"` or `data-icon="inline-end"`.
  - `Badge`: variants `default` (quiet: `bg-muted text-foreground`) `| secondary | destructive | outline | ghost | link`.
  - `Card`: `size="sm"` (renders `data-size="sm"`) for compact spacing; compose `CardHeader`, `CardTitle`, `CardDescription`, `CardAction`, `CardContent`, `CardFooter`. A card is a discrete object, never a photo stage.
  - `Alert`: variants `default | destructive`; destructive renders as a leading `border-l-2 border-l-destructive` rule. Compose `AlertTitle`, `AlertDescription`, `AlertAction`.
  - `Separator`: hairline `bg-border`, horizontal by default.
- Foundations stories: `Foundations/Typography` and `Foundations/Atmospheres`.
- Already applied by the base layer in `globals.css`; do not redeclare: `border-border` and `outline-ring/50` on all elements, `font-sans text-sm antialiased` on `html`, `bg-background text-foreground` on `body`.

If nothing fits, compose semantic HTML plus tokens. Never invent parallel class systems. Arbitrary values only for established sizes (`text-[0.8125rem]`).

## Accessibility and keyboard

- Meet WCAG AA contrast in both atmospheres. Never rely on color alone; this is why Technical Check Levels are shape plus text.
- Keep Technical Check Levels visibly distinguishable and clearly named in both atmospheres.
- Keep focus visible through the ring token (the `focus-visible:ring-ring/50` pattern), keep semantic landmarks, and keep source order equal to reading order.
- Respect `prefers-reduced-motion` wherever motion exists.
- Photart is a desktop windowed app: layouts must hold at the window's minimum size without horizontal scrolling. No mobile-breakpoint thinking.

The target is studio-assistant judgment, not decoration. The photograph is the interface.
