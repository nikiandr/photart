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

- The design system has two layers. Reuse quiet shared primitives and invest in photographic patterns such as the Photo Stage, Technical Check marker, Burst Comparison, Shoot entry, Thread workspace, and Handoff. These patterns protect attention, momentum, and authorship.
- Extend the existing shared components and reference the named tokens in `src/styles/globals.css`. Use the existing atmosphere system for Gallery and Review.
- Give each new component a Storybook story. Use the Typography and Atmospheres foundations as references.

## Work in four passes

### 1. Frame the photographer's moment

Ask these privately before designing anything:

- Which atmosphere owns this moment: Review (darkroom concentration, one luminous photograph) or Gallery (photobook and contact-sheet reflection, spacious rediscovery)?
- What decision is the photographer making right now?
- What is the minimum information that serves that decision? Move everything else behind progressive disclosure.
- How does a fluent photographer repeat this action hundreds of times by keyboard?

For the full evaluation frame, see the nine Decision Tests in `docs/design/design-philosophy.md`.

### 2. Choose the composition

- Place the photograph on the stage and let a quiet frame surround it.
- Compose sections with space and hairline rules. Reserve cards for discrete objects.
- Keep one focal point per screen. Align aggressively. Prefer square, barely-softened geometry.
- Do not force photographs into identical containers; layouts holding photographs may respond to aspect ratio, sequence, and rhythm.
- Practice informational and attentional minimalism, not merely visual minimalism: show what the photographer needs now, make deeper evidence available in place, then get out of the way.
- The frame is disciplined; photography is free. Keep controls, states, and behavior consistent, and never let a screen resemble a component catalog.

Composition is the system; decoration is not.

### 3. The Photart visual system

Treat this section and `src/styles/globals.css` as the design authority. Do not introduce a parallel visual system, ad-hoc values, or one-off overrides.

#### Atmospheres

Two related atmospheres, one system:

- Gallery uses light, warm paper neutrals with a contemporary feel. Avoid cream and sepia.
- Review uses cool, deep, low-chroma neutrals so photographs stay the strongest color source.

Rules:

- Apply the atmosphere to the full surface that owns the moment.
- Never blend atmospheres on one surface. One surface, one atmosphere.
- Keep typography, interaction behavior, and component grammar identical across both, so the two environments remain unmistakably related rather than separate applications or modes.
- The atmosphere follows the work along one continuous flow (Import Shoot, Technical Check, Review Shoot, Gallery, Handoff). It is an environment change, never a module switch.
- Use Storybook to compare both atmospheres.

#### Geometry, spacing, and alignment

- Keep geometry sharp and composed with a consistent, barely softened corner radius.
- Use borders and hairline rules over shadows; use space over container proliferation.
- Use the existing spacing scale.
- Do not use plush radii, pills, or soft floating panels.
- Inside data sections, use thin rules to separate rows.

#### Typography and rhythm

A frozen five-rung ladder. Add no other sizes without defining a new role first:

- Use Newsreader for editorial display, including Shoot names, Gallery headings, and sparse reflective moments. Keep dense interface text in IBM Plex Sans.
- Set headings in prominent IBM Plex Sans.
- Set subheadings in smaller IBM Plex Sans.
- Set body copy in IBM Plex Sans with comfortable line spacing and mute supporting text.
- Use compact interface text and IBM Plex Mono for EXIF, filenames, dimensions, counts, and shortcuts. Keep surrounding sentences in sans.

In definition lists, use muted sans labels and mono values. See the Typography foundation in Storybook.

IBM Plex Sans is the interface default. Use Newsreader as a deliberate editorial accent.

#### Color, surfaces, and boundaries

- Low-chroma neutrals only; no fixed meaning is ever tied to a specific hue.
- Use color as an interface accent only for irreversible consequences such as Purge. Keep warnings and emphasis neutral.
- Keep ordinary badges muted. Mark a destructive alert with a thin leading rule.
- Use a muted badge for a Suggested Frame, an outlined badge for All Good, and a monospaced badge for bare values such as ISO. See the Atmospheres foundation in Storybook.
- Pair each Technical Check Level (All Good, Minor Issues, Significant Issues) with a distinct muted symbol and its text label. Never use traffic-light hues or color alone. The levels describe the amount and significance of findings. Confidence and artistic quality are outside that scale.

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

- Budget 150–250 ms. Motion must never delay the next judgment or compete with the photograph.
- Permitted uses: connecting a thumbnail to focused inspection, preserving spatial understanding between related states, helping compare imagery.
- Atmosphere switches are crossfade-only; the stage never layout-shifts.
- Honor reduced-motion preferences for every transition.

#### Media and icons

- The photograph is the only hero. No decorative imagery, illustrations, or stock assets, ever.
- Use the existing icon set and align icons visually with nearby text.
- An icon must make the action materially faster to recognize; otherwise use a text label. Never use emoji as icons.
- Interpret photography contemporarily: learn from framing, sequencing, contact-sheet rhythm, and negative space, but never imitate analog artifacts (film grain, retro camera chrome, darkroom skeuomorphism).
- Render photographs at their own aspect ratio. Preserve the full frame in the grid and on the stage.

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
- Render both atmospheres.
- Walk the entire flow keyboard-only, and confirm every Review action shows its shortcut in mono.
- Check reduced-motion behavior.
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

## Use existing tokens and components

Use the existing token system and shared components. Look up token names in `src/styles/globals.css` and component options in `src/components/ui/`. If no shared component fits, compose semantic HTML with existing tokens. Keep new work within the same visual system.

## Accessibility and keyboard

- Meet WCAG AA contrast in both atmospheres. Never rely on color alone; this is why Technical Check Levels are shape plus text.
- Keep Technical Check Levels visibly distinguishable and clearly named in both atmospheres.
- Keep keyboard focus visible, retain semantic landmarks, and keep source order aligned with reading order.
- Respect reduced-motion preferences wherever motion exists.
- Photart is a desktop windowed app: layouts must hold at the window's minimum size without horizontal scrolling. No mobile-breakpoint thinking.

The target is studio-assistant judgment, not decoration. The photograph is the interface.
