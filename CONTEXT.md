# Photart

Photart is a review-first photography workbench that keeps completed work available for later browsing, comparison, and thematic rediscovery.

## Language

**Photographer**:
The artist whose creative intent governs every selection and rejection in Photart. Being an amateur describes the photographer's practice, not the value or seriousness of their work.
_Avoid_: User, operator, content creator

**Creative Intent**:
The photographer's subjective reason for making or keeping a photograph, which may deliberately override technical conventions.
_Avoid_: Correctness, quality score

**Studio Assistant**:
Photart's product posture: it quietly carries mechanical work, presents observations without judgment, and waits for the photographer to make creative decisions. Guidance is available on request rather than imposed.
_Avoid_: Teacher, critic, judge, autopilot

**Catalog**:
A persistent collection of photographs from multiple shoots, represented by a user-visible folder on the filesystem. A catalog remains available in Photart for later exploration and supplies selected photographs to downstream editing or publishing workflows.
_Avoid_: Library, database, album

**Source**:
The external folder selected for import, commonly located on an SD card.
_Avoid_: Import folder, input, origin

**Import**:
The creation of catalog-owned copies of every photograph in a source before Review begins.
_Avoid_: Open, scan, sync

**Shoot**:
A named group of photographs imported together and represented by a folder inside the catalog. Importing a source creates one Shoot.
_Avoid_: Session, project, collection, batch

**Burst**:
A sequence of closely timed photographs of the same moment or scene that belongs together for rapid comparison during Review.
_Avoid_: Duplicates, stack, batch

**Suggested Frame**:
The photograph Photart proposes as a useful starting point when comparing a Burst, accompanied by visible reasons. It is never automatically added to the Selection.
_Avoid_: Best photo, winner, automatic pick

**Technical Check**:
An automated first pass that groups photographs by the amount and significance of detected technical issues, such as missed focus or problematic exposure. It prioritizes human review without judging artistic worth.
_Avoid_: Triage, quality rating, culling, good/bad classification

**Technical Finding**:
An evidence-backed observation produced by Photart about a photograph, such as the likely focus area or an exposure warning. A finding may describe an intentional artistic choice and is never itself a rejection.
_Avoid_: Flaw, failure, verdict

**Contextual Analysis**:
A concise, model-assisted explanation that connects technical evidence with visible photographic context. It may suggest possible interpretations but never claims to know the photographer's intent, and it appears within Review rather than as a separate conversation.
_Avoid_: Chat, critique, quality score

**Technical Check Level**:
A three-level summary based on the amount and significance of a photograph's technical findings: All Good, Minor Issues, or Significant Issues. It does not express Photart's confidence or the photograph's artistic value.
_Avoid_: Issue likelihood, quality score, good/bad rating

**Review**:
The photographer's process of evaluating a group of photographs and deciding which are worth carrying forward.
_Avoid_: Culling, triage, cleaning, filtering

**Rejection**:
A reversible decision made by the photographer that excludes a photograph from the selection. The Technical Check can propose closer review but can never create a rejection.
_Avoid_: Delete, trash, remove

**Archive**:
A reversible disposition that retains a rejected photograph in the catalog while hiding it from normal browsing.
_Avoid_: Delete, purge, reject

**Purge**:
An explicit disposition that permanently deletes the files of rejected photographs from a catalog.
_Avoid_: Reject, clean up

**Selection**:
A group of photographs deliberately chosen by the photographer for a downstream workflow.
_Avoid_: Favorites, accepted photos, good photos

**Handoff**:
The transfer of a Selection from Photart into a downstream editing, publishing, printing, or presentation workflow without altering the original photographs.
_Avoid_: Export edit, develop, publish

**Gallery**:
The reflective browsing experience reached after Review, where the photographer can revisit selected photographs across the catalog and rediscover themes.
_Avoid_: Develop view, library module

**Thread**:
A named thematic relationship the photographer creates across photographs from different Shoots. Photart can help search by meaning, palette, place, scene, motion, framing, or other visible qualities, but the photographer initiates, authors, and may deliberately sequence the Thread.
_Avoid_: Album, automatic collection, suggested theme
