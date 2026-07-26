# Photart Design Philosophy

**Status:** Accepted
**Date:** 2026-07-26

Photart is a review-first photography workbench for amateur photographers. It helps a photographer move from an overwhelming Shoot to a deliberate Selection without competing with the art or claiming authority over it.

This document defines how Photart should feel and how its design system should make decisions. Domain terminology lives in [CONTEXT.md](../../CONTEXT.md); the fuller reasoning behind these principles lives in [Design Philosophy Discussion](./design-philosophy-discussion.md).

## North Star

### Release the photograph from the noise

The photograph already exists. Photart helps release it from the volume, repetition, uncertainty, and mechanical work surrounding it.

A Shoot may begin as 900 files. Photart helps the photographer arrive at 100 photographs worth keeping, 50 strong photographs, and perhaps five that demand a further life in editing, print, publication, exhibition, a book, or an album cover.

The outcome should feel like a burden has been lifted: the pile has become understandable, the work has become approachable, and the photographer still has creative energy for what comes next.

## Product Posture

### Be a quiet studio assistant

Photart quietly carries mechanical work, presents useful observations, and waits for the photographer to decide. It is calm, capable, and present without becoming the center of the process.

Empathy is not a friendly visual treatment. It is respect for artistic agency:

- reduce clerical and cognitive burden;
- protect momentum;
- make uncertainty understandable;
- preserve reversibility;
- keep creative decisions with the photographer.

Being an amateur does not make the photographer's artistic judgment less serious. Photart must never adopt a superior posture toward the person or the work.

## Governing Principles

### 1. Artistic judgment comes first

Technical conformity is supporting evidence, not artistic truth.

Photart may identify missed focus, clipped exposure, motion, or another technical concern. A concern may also be deliberate, expressive, or essential to the photograph. Photart can direct attention and explain evidence, but only the photographer may Select, Reject, Archive, or Purge.

### 2. Create one continuous photographic flow

The experience follows one connected movement:

**Import Shoot → Technical Check → Review Shoot → Gallery → Handoff**

These stages are not Lightroom-style modules that require repeated back-and-forth navigation. The visual environment changes as the work changes, while the photographer remains inside one continuous process.

### 3. Provide intelligence in context

Photart's intelligence appears as concise, embedded Contextual Analysis—not as a separate conversation.

It may connect technical evidence with visible subject matter, composition, neighboring frames, and photographic context. It may offer possible interpretations, but it never claims to know the photographer's intent.

The system should be perceptive without becoming presumptuous.

### 4. Create focus without isolation

One photograph dominates during Review, but it never becomes disconnected from its sequence.

Adjacent frames, Bursts, and closely related alternatives remain immediately available. A Suggested Frame may provide a reasoned starting point for comparing a Burst, but it is not an automatic choice or an artistic verdict.

### 5. Protect momentum

Momentum is a form of empathy. Once Review begins, navigation, comparison, selection, rejection, inspection, and reversal should feel immediate.

The design should support a beginner visibly and become faster with familiarity:

- repeated actions have stable, memorable shortcuts;
- every action also remains available to a pointer;
- shortcuts accelerate fluency without becoming hidden requirements;
- the interface saves decisions continuously;
- reversible actions happen immediately;
- confirmation is reserved for genuinely irreversible consequences.

### 6. The frame is disciplined; photography is free

Photart's interface follows a strict, reusable grammar. Photographic composition does not.

Controls, typography, spacing logic, states, and behavior should remain consistent. Layouts containing photographs may respond to aspect ratio, sequence, rhythm, and visual relationships. The system must not force every photograph into identical cards or make every screen resemble a component catalog.

Minimalism is informational and attentional, not merely visual. Show what the photographer needs now; make deeper evidence available in place; then get out of the way.

### 7. Interpret photography contemporarily

Photart is a tool for digital photography. It may learn from photographic behaviors—framing, sequencing, contact-sheet rhythm, negative space, focus, and careful observation—without imitating analog artifacts or retro equipment.

The interface should feel made by people who understand photography, not decorated with photography symbols.

### 8. Use two related atmospheres in one system

Review and Gallery express different mental states without becoming separate applications or modes:

- **Review** has the concentration of a darkroom: restrained, quiet, and centered on one luminous photograph.
- **Gallery** has the reflective openness of a photobook or contact sheet: structured, spacious, and conducive to rediscovery.

Typography, interaction behavior, and component grammar keep the two environments unmistakably related.

### 9. Speak calmly and precisely

Photart's voice is calm, observant, quietly encouraging, and honest about uncertainty.

Prefer short, concrete language:

- “Your Shoot is ready to review.”
- “Focus may fall behind the subject.”
- “12 photographs selected.”
- “Continue when you're ready.”

The product can help someone learn through clear observations, but it does not assume the posture of a teacher. Understanding should emerge naturally from useful explanations.

### 10. Support authored rediscovery

Shoots preserve where photographs came from. Threads express what the photographer sees across them.

The photographer initiates a Thread and may search by meaning, palette, place, scene, motion, framing, or another visible quality. Photart supplies search materials and relationships; it does not push themes as authored conclusions.

A photograph may belong to several Threads. The photographer may deliberately sequence a Thread so it can become the beginning of a series, book, exhibition, or album concept.

### 11. Stop before pixel editing

Photart imports, analyzes, compares, organizes, reviews, archives, and hands off photographs. It does not develop, retouch, or generate imagery.

Its responsibility is to help reveal strong work and carry the Selection into the next creative workflow without substituting software output for photographic authorship.

## Design-System Approach

### Two layers

The system has two complementary layers:

1. **Quiet primitives** provide reliable, accessible behavior for buttons, menus, fields, dialogs, and other controls. The installed shadcn/Base UI foundation is a chassis, not Photart's identity.
2. **Photographic patterns** encode the product philosophy. Examples include the Photo Stage, Technical Check marker, Burst Comparison, Shoot entry, Thread workspace, and Handoff.

The value of the design system is not the number of generic components it contains. Its value is how clearly its photographic patterns protect attention, momentum, and authorship.

### Visual grammar

Photart is sharp and composed rather than plush or rounded:

- square or barely softened geometry;
- strong alignment and quiet rules;
- space instead of container proliferation;
- visual character through composition, typography, rhythm, and restraint;
- no fixed requirement that meaning depend on a specific hue.

Technical Check Levels must remain visibly distinguishable and clearly named:

- **All Good**
- **Minor Issues**
- **Significant Issues**

These labels describe the amount and significance of technical findings, not confidence and never artistic quality. Color, symbols, borders, ordering, and density may support the distinction, but the philosophy does not prescribe exact element-level tokens.

### Typography

The working type system joins editorial warmth with technical precision:

- **Newsreader** for Shoot names, Gallery headings, and sparse reflective moments;
- **IBM Plex Sans** for controls, navigation, findings, and explanatory copy;
- **IBM Plex Mono** only for EXIF, filenames, counts, and shortcuts.

The serif is a deliberate accent, not a decorative layer applied to dense interface text.

### Motion

Motion follows one rule:

> Motion preserves continuity, reveals causality, or assists inspection. If it does none of those, it does not move.

Motion may connect a thumbnail to focused inspection, preserve spatial understanding between related states, or help compare imagery. It must not delay the next judgment or compete with the photograph.

### Progressive mastery

Photart uses one coherent interface, not separate Beginner and Advanced modes.

Clear defaults and obvious primary actions make the first Shoot understandable. Comparison, Contextual Analysis, shortcuts, and deeper organization become discoverable in context as the photographer grows fluent.

## Technical Check and Human Authority

The Technical Check groups photographs by the amount and significance of detected technical concerns:

1. **All Good** — no meaningful concern detected.
2. **Minor Issues** — limited concerns worth awareness.
3. **Significant Issues** — substantial or multiple concerns worth closer inspection.

The levels organize attention. They do not create a Selection or Rejection.

At the end of Review:

- Rejection remains a reversible photographer decision;
- Archive retains rejected work while hiding it from normal browsing;
- Archive is the default disposition;
- Purge is an explicit, secondary decision;
- prompts refer to “rejected photographs,” never ambiguously “filtered photos.”

## Decision Tests

When evaluating a design direction, ask:

1. Does it protect the photographer's authority?
2. Does it reduce burden without reducing creative choice?
3. Does the photograph remain the strongest presence?
4. Does it preserve momentum and make reversal safe?
5. Is intelligence embedded, contextual, and honest about what it knows?
6. Does the interface communicate clearly on first encounter and reward familiarity later?
7. Does motion explain a relationship or assist inspection?
8. Does the system feel contemporary and photographic without imitation?
9. Does this belong to the quiet frame, or is it competing with the work?

## Success

Photart succeeds when the photographer finishes Review feeling lighter, clearer, and closer to their own work.

The mass of files has become a deliberate Selection. Technical uncertainty has become understandable context. A few photographs have emerged as work worth carrying forward. The achievement belongs to the photographer, and Photart recedes into the background.

## Related Decisions

- [Preserve human authority during culling](../adr/0001-preserve-human-authority-during-culling.md)
- [Use a continuous photographic workflow](../adr/0002-use-a-continuous-photographic-workflow.md)
- [Stop before pixel editing](../adr/0003-stop-before-pixel-editing.md)
