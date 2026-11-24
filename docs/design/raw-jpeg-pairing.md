# RAW + JPEG in Photart

A RAW file and its matching camera JPEG are two originals of one photograph. Photart keeps both and treats the pair as one photograph for Review. A Burst, by contrast, contains distinct exposures that can be judged separately.

## Pairing

Photart pairs one recognized RAW with one JPEG when the files have the same filename stem in the same Source folder. Ambiguous matches and files with conflicting reliable capture metadata remain separate. The photographer can unlink a pair without deleting either file and undo that change.

## Gallery and Detail

Gallery shows the pair as one tile by default, using the RAW preview as the cover. Clicking the image opens Detail on the RAW. A separate, keyboard-accessible `RAW + JPEG` toggle expands the pair into adjacent, labeled thumbnails and collapses it back to one tile. Opening either expanded thumbnail opens Detail on that format.

Detail and Review offer a `RAW / JPEG` switch for the current photograph. Switching to JPEG does not change the browsing default. Previous, Next, and filmstrip navigation move between photographs, not between files, and open each paired photograph on RAW. Returning to a previously viewed pair also opens RAW. A standalone photograph opens in its own format. If the RAW preview is unavailable and the JPEG can be shown, Photart labels the fallback explicitly.

## Review and Handoff

Review counts the pair once. Selection, Rejection, and Archive apply to the photograph, regardless of which format is visible. A Technical Finding identifies the format it describes. At Handoff, the photographer chooses RAW, JPEG, or both; the Gallery cover does not determine which originals are sent. Photart reports missing originals rather than silently substituting another format. Purge identifies the exact originals it will permanently delete.

## Rationale

Review should ask whether an exposure is worth carrying forward, not ask the photographer to make that decision twice. RAW-first browsing keeps the Gallery cover and the default Detail view consistent. The companion JPEG preserves the in-camera rendering and is available for inspection or Handoff without changing how the photographer moves through the Shoot.
