# Photart Design Philosophy Discussion

**Status:** Decision record
**Date:** 2026-07-26

This document preserves the important questions, corrections, alternatives, and decisions that produced the focused [Photart Design Philosophy](./design-philosophy.md). It intentionally contains more context than the philosophy so future contributors can understand not only what was chosen, but what the choice was trying to protect.

## Starting Point

Photart began as an idea for amateur photographers facing a familiar burden: an SD card or folder containing hundreds of photographs of uneven technical quality. The desired product would copy the photographs into a persistent catalog, analyze them, support rapid review and comparison, help remove or archive unwanted work, and retain selected photographs for later rediscovery and handoff.

The photographer references were Anton Corbijn, Pixy Liao, and Lars Tunbjörk. They were not supplied as a visual style to imitate. They expressed a preference for:

- direct emotional connection;
- a clear subject or object;
- structural density without visual chaos;
- purposeful details;
- intimacy, imperfection, tension, color, and human observation;
- photographs that communicate on first encounter without requiring visual excavation.

The discussion translated those preferences into product behavior and design posture rather than photographic effects.

## Product and Domain Decisions

### 1. Is Photart temporary or permanent?

**Question:** Is Photart a temporary culling utility that hands work to another library, or a permanent home for photographs?

**Decision:** Photart is a persistent home and a review-first workbench. A Catalog corresponds to a user-visible folder and remains available for later browsing, comparison, and thematic rediscovery. A Selection can still be handed to downstream tools.

**Reason:** The photographer wants to return to older work and discover relationships that were not visible during the original Shoot.

### 2. What happens when an SD card or folder is chosen?

**Question:** Review photographs in place and copy only the Selection, or copy everything first?

**Decision:** Import every source photograph into the Catalog before Review.

**Reason:** The Catalog becomes the safe working copy, Review can continue after the card is removed, and the source is never the only copy being manipulated.

### 3. What is the unit of import?

**Question:** Should Photart automatically split an import into sessions?

**Decision:** Import one Shoot. A Shoot is a named folder inside the Catalog containing photographs imported together. Automatic splitting is not part of the current model.

**Context:** The photographer currently organizes folders with a number, date, and description. That exact naming scheme is personal and was not adopted as a system requirement, but the term Shoot matched the photographer's mental model.

### 4. Is the workflow modular?

**Question:** Should Review and Gallery behave like separate Lightroom modules?

**Decision:** No. The flow is continuous:

**Import Shoot → Technical Check → Review Shoot → Gallery → Handoff**

Review and Gallery may feel different, but the photographer should not repeatedly switch between application modules.

### 5. Does Photart edit photographs?

**Question:** Should Photart eventually include pixel-level Develop or generative tools?

**Decision:** No. Photart imports, analyzes, compares, organizes, reviews, archives, and hands off. It does not develop, retouch, or generate imagery.

**Reason:** The art is made in the photographic act and subsequent workflows chosen by the photographer. Photart's role is to reveal work from the surrounding volume and carry it forward.

## Artistic Authority and Safety

### 6. Does technical analysis decide what is good?

**Question:** Are the three groups judgments of good, mediocre, and bad photographs?

**Decision:** No. They summarize technical concerns from Photart's perspective. A technically unconventional photograph may be artistically essential.

**Principle:** Artistic judgment is first; technical conformity is second.

### 7. Can Photart automatically reject?

**Question:** May technical analysis ever Select, Reject, Archive, or Purge a photograph?

**Decision:** Never. Photart may organize attention, highlight concerns, and make proposals. Only the photographer changes the creative status of a photograph.

### 8. What are the three technical groups?

The categorization was initially discussed through red, yellow, and green. The important decision was semantic distinction, not literal traffic-light colors.

Several labels were rejected:

- “Good / So-so / Bad” confused technical condition with artistic worth.
- “Clear / Review / Inspect” did not create a clear severity hierarchy.
- “No issue / Possible issue / Likely issue” incorrectly made the scale about model confidence.

**Final decision:**

- **All Good**
- **Minor Issues**
- **Significant Issues**

The heading **Technical Check** limits “All Good” to technical analysis. The scale summarizes the amount and significance of findings; uncertainty belongs inside individual findings.

Familiar green–yellow–red direction may inform visual exploration, but the philosophy does not fix exact colors. States must be distinguishable through more than hue alone.

### 9. Is rejection deletion?

**Question:** Should a rushed reject action remove a file?

**Decision:** No. Rejection is a reversible photographer judgment. It is distinct from file disposition.

### 10. What happens to rejected photographs?

**Decision:** At the end of Review, Photart asks what should happen to the rejected photographs:

- **Archive** keeps the files in the Catalog and hides them from normal browsing.
- **Purge** permanently deletes their files.

Archive is the default. Purge is secondary and explicit. Copy must refer to a concrete count of “rejected photographs,” never ambiguously to “filtered photos,” because a technical filter is not a rejection.

### 11. How broadly should reversibility apply?

**Decision:** Reversibility governs the experience. Ordinary organizational and review actions save continuously and remain undoable. Confirmation is reserved for genuinely irreversible consequences.

## Product Character

### 12. What did the photographer references contribute?

**Question:** Should the interface imitate grain, hard contrast, expressive color, or another surface quality from the references?

**Decision:** No. The references inform deeper values: reduction, directness, human imperfection, intimacy, tension, composition, and attentive observation.

The interface protects those qualities in the photographer's work rather than reproducing them as decoration.

### 13. What relationship should Photart have with the photographer?

**Decision:** Photart behaves as a quiet studio assistant.

It carries mechanical work, presents observations without judgment, waits for creative decisions, and makes deeper explanation available in context. Empathy means respect for agency, not a soft or overly friendly visual style.

### 14. Should Photart teach?

**Decision:** Photart does not assume the posture of a teacher. Learning may emerge from clear explanations and Contextual Analysis, but guidance is calm, contextual, and available when useful.

### 15. What should the photographer feel at the end?

**Decision:** Lighter, clearer, and closer to their own work.

The intended emotional transformation is:

- from 900 undifferentiated files to a manageable body of work;
- from technical uncertainty to understandable context;
- from burden to creative possibility;
- from many photographs to a Selection and a small number of exceptional frames worth developing further.

The achievement belongs to the photographer, not the software.

## Intelligence and Analysis

### 16. How should intelligent models appear?

**Decision:** Intelligence is embedded in the photographic workflow as Contextual Analysis. It is not a separate chat experience.

It may connect focus, exposure, composition, subject, motion, and neighboring frames to useful photographic context. It should provide more than mechanical measurement while remaining concise and actionable.

### 17. May analysis infer creative intent?

**Decision:** It may suggest possibilities but never claim intent.

Preferred form:

> “The directional blur follows the subject's movement and may be intentional.”

Rejected form:

> “You intended this motion blur.”

The system should feel perceptive, not presumptuous.

### 18. How should Bursts work?

**Decision:** Closely timed photographs of the same moment form a Burst and remain available for rapid comparison. Review focuses on one frame without isolating it from the sequence.

### 19. Can Photart recommend a frame?

**Decision:** Yes, as a **Suggested Frame**, not a “Best Photo.”

The suggestion may consider technical and visible contextual evidence, must show its reasons, and never automatically joins the Selection. It offers a useful place to begin comparing rather than an artistic verdict.

## Review Experience

### 20. Dashboard or viewing space?

**Decision:** Review is an uninterrupted viewing space. One photograph dominates. Technical findings, metadata, and controls remain quiet until relevant.

Contact sheets, detailed evidence, and comparisons are intentional secondary states rather than permanent visual competition.

### 21. Is a photograph reviewed alone?

**Decision:** Use **focus without isolation**. Keep neighboring frames, Bursts, and related alternatives close enough for immediate comparison.

### 22. How much information appears?

**Decision:** Minimalism applies to information and attention. Show only what matters to the current judgment. Deeper detail should be shallow, direct, understandable on first encounter, and dismissed without breaking rhythm.

The discussion used a background chooser as an example of this interaction pattern, not as a request to specify every element-level token.

### 23. What role does speed play?

**Decision:** Momentum is a form of empathy. Import and analysis may require time, but Review itself should feel immediate.

Motion may clarify a change but must not delay the next judgment.

### 24. Keyboard or pointer?

**Decision:** Review is keyboard-led and pointer-complete.

Repeated actions receive stable, memorable shortcuts. Every action remains visibly discoverable and operable with a pointer. Shortcuts reward fluency without becoming hidden requirements.

### 25. Beginner and Advanced modes?

**Decision:** Use one coherent interface with progressive mastery.

The first Shoot should be understandable through clear defaults and visible actions. Deeper comparison, Contextual Analysis, shortcuts, and organization appear naturally in context as the photographer grows fluent.

## Catalog, Gallery, and Threads

### 26. What is the Gallery for?

**Decision:** Gallery is a reflective place for revisiting selected work after Review. It supports later discovery without becoming a Develop module.

### 27. Should Photart proactively invent themes?

**Decision:** No. The photographer initiates the search for a Thread.

Photart may help search by:

- semantic meaning;
- palette;
- place or available location metadata;
- scene;
- motion;
- framing;
- other visible qualities.

The photographer decides whether the relationship is meaningful and creates the Thread.

### 28. Can a photograph belong to several Threads?

**Decision:** Yes. Threads are interpretive lenses, not filesystem folders. One photograph may participate in several thematic readings without being copied or moved.

### 29. Can Threads be sequenced?

**Decision:** Yes. The photographer's saved order is authoritative. Sequencing allows a Thread to become the beginning of a series, book, exhibition, or album concept without turning Photart into a publishing tool.

## Visual and Verbal System

### 30. How rigid is the system?

**Decision:** The frame is disciplined; photography is free.

Interface grammar remains strict and reusable. Photographic layouts may respond to aspect ratio, sequence, rhythm, and relationships. Photographs should not be forced into identical generic cards.

### 31. Analog or contemporary?

**Decision:** Contemporary digital photography.

Photart may borrow photographic behaviors such as framing, sequencing, contact-sheet rhythm, focus, and negative space. It should not rely on analog imitation or retro equipment metaphors to appear photographic.

### 32. One atmosphere or two?

**Decision:** Two related atmospheres inside one continuous system:

- Review has the concentration of a darkroom.
- Gallery has the reflective openness of a photobook or contact sheet.

These are experiential shifts, not application modules.

### 33. Soft or sharp geometry?

**Decision:** Sharp and composed.

Prefer strong alignment, quiet rules, open space, and square or barely softened geometry. Empathy comes from clarity, pacing, language, and behavior rather than plush shapes or container-heavy layouts.

### 34. What typography should Photart use?

**Decision:**

- **Newsreader** for sparse editorial moments;
- **IBM Plex Sans** for interface language;
- **IBM Plex Mono** for EXIF, filenames, counts, and shortcuts.

The current all-monospace prototype reads as a technical inspector. The chosen system introduces photobook warmth without losing precision.

### 35. What is the voice?

**Decision:** Calm, observant, quietly encouraging, concise, and honest about uncertainty.

Examples:

- “Your Shoot is ready to review.”
- “Focus may fall behind the subject.”
- “12 photographs selected.”
- “Continue when you're ready.”

## Component-System Direction

### 36. Keep or replace the installed component system?

At the time of discussion, the project used shadcn v4's Base Lyra style on Base UI, Tailwind CSS variables, Phosphor icons, and Storybook.

**Decision:** Evolve it into two layers:

1. **Quiet primitives** retain reliable behavior and provide a stable chassis.
2. **Photographic patterns** carry Photart's identity and encode domain behavior.

The installed component styling is not sacred, but replacing working behavioral foundations without reason is unnecessary.

### 37. How should motion be judged?

**Decision:**

> Motion preserves continuity, reveals causality, or assists inspection. If it does none of those, it does not move.

This rule permits expressive interaction while keeping the photograph dominant.

### 38. Which external component ideas were considered?

No components were installed during the discussion.

| Candidate | Decision context |
| --- | --- |
| Motion Primitives Morphing Dialog | Strong candidate for spatial continuity from thumbnail to focused inspection. Adapt the behavior while retaining audited dialog semantics. |
| Motion Primitives Carousel | Strong candidate for Burst navigation mechanics. It should become a keyboard-first photographic pattern, not retain generic website-carousel presentation. |
| Kibo UI Image Zoom | Strongest initial candidate for fullscreen inspection because its documentation includes keyboard, touch, controlled state, and reduced-motion behavior. |
| Magic UI Lens | Conditional candidate for inspecting a suspected focus region; overlaps with fullscreen zoom and requires interaction review. |
| Motion Primitives Progressive Blur | Narrow use around temporary chrome only; never obscure a photograph being judged. |
| Motion Primitives In View | Narrow use for occasional structural transitions, not repeated animation of every Gallery image. |
| Magic UI Blur Fade | Do not adopt initially; blurring photographs during reveal weakens visual honesty and scanning speed. |

Motion Primitives was labeled beta at the time of review. Candidate registry components should be treated as source material Photart owns and adapts, not as external visual authority.

## Terminology Decisions

### 39. Culling, triage, or review?

“Culling” is established photography-industry language but may be unfamiliar to amateurs. “Triage” has useful analogies but is not the clearest user-facing photography term.

**Decision:** Photart uses plain product language:

- Photart performs a **Technical Check**.
- The photographer **Reviews a Shoot**.
- The photographer **Selects** or **Rejects**.

“Culling” may remain useful in industry or strategy discussion but is not required product vocabulary.

## Decisions That Are Not Philosophy Pillars

Several topics were agreed as implementation considerations but explicitly excluded from the central design philosophy:

### Local processing

Local-first behavior is a sensible trust default. Remote analysis, if ever used, should be explicit. It is not a headline identity or marketing pillar.

### Accessibility

The interface should be accessible, but accessibility was not selected as a defining philosophical pillar. It remains an implementation quality standard.

### Current development scope

The narrow initial scope reflects the practical reality of a solo developer. It is not an aesthetic or design-system principle and must not be presented as one.

## Final Shared Understanding

Photart exists to remove the burden between taking photographs and recognizing the work worth carrying forward.

It is not the artistic process itself. It protects that process by remaining quiet, responsive, observant, and subordinate to the photographer's judgment. Its design system succeeds when the interface frames the work clearly, its intelligence adds context without claiming authority, and a large, intimidating Shoot becomes a Selection the photographer is excited to develop, print, publish, exhibit, or sequence.

## Reference Links

### Photography references

- [Anton Corbijn interview on intimacy and imperfection](https://time.com/4103961/the-secret-behind-anton-corbijns-signature-look/)
- [Pixy Liao profile at Aperture](https://aperture.org/editorial/pixy-liao-jon-feinstein/)
- [Lars Tunbjörk Foundation](https://tunbjork.se/)

### Typography

- [Newsreader by Production Type](https://productiontype.com/font/newsreader)
- [IBM Plex](https://github.com/IBM/plex)

### Component candidates

- [Motion Primitives Morphing Dialog](https://motion-primitives.com/docs/morphing-dialog)
- [Motion Primitives Carousel](https://motion-primitives.com/docs/carousel)
- [Motion Primitives Progressive Blur](https://motion-primitives.com/docs/progressive-blur)
- [Motion Primitives In View](https://motion-primitives.com/docs/in-view)
- [Magic UI Lens](https://magicui.design/docs/components/lens)
- [Magic UI Blur Fade](https://magicui.design/docs/components/blur-fade)
- [Kibo UI Image Zoom](https://www.kibo-ui.com/components/image-zoom)
