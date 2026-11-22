# Photo import formats and decoders

## What Photart imports

| Files | What is displayed |
| --- | --- |
| `.jpg`, `.jpeg`, `.png`, `.webp` | The existing JPEG thumbnail and original-file detail view. |
| `.heic`, `.heif` | The primary still image, including tiled grids supported by the decoder. The HEIF file must have a `heic`, `heix`, or `mif1` major brand and an 8-bit intra-coded HEVC image in 4:2:0 or monochrome. AVIF, other codecs, 10-bit HEIC, and sequence playback are not supported. HEIF crop, rotation, and mirroring are applied by the decoder; EXIF orientation is not applied a second time or used as a fallback. Depth, gain maps, and secondary images are not rendered. |
| `.tif`, `.tiff` | The first page of a classic TIFF or BigTIFF, with TIFF orientation applied. Common grayscale, RGB(A), and 16-bit RGB TIFFs work; decoding is limited to the colors and compressions implemented by the `image` TIFF adapter and `tiff` crate. Not all TIFF variants work (for example, the adapter does not handle palette TIFF). Multipage files are not split into separate photographs. |

Import checks the content as well as the extension. TIFF's container is also
used by some camera RAW files: Photart rejects DNG version tags, CFA tags,
raw photometric interpretations, and the Canon CR2 header rather than
importing an embedded preview as a photograph. It checks the TIFF image
directories and SubIFDs (up to 64 directories, rejecting cycles or malformed
links). Proprietary RAW files are not supported; these checks are not a
general-purpose RAW classifier for every renamed proprietary format.

Inputs are capped at 100 MiB and 100 megapixels, with an additional 10,000-pixel
per-axis HEIC limit and the underlying decoder's allocation and format limits.
HEIC and TIFF decodes run one at a time even during a parallel import. Originals
stay byte-for-byte intact. Their JPEG display copies are 8-bit RGB and don't
retain extra channels or apply full ICC/HDR color management. Photart reads the same camera,
lens, capture time, aperture, shutter speed, and ISO fields from original EXIF
when the metadata reader can parse it. EXIF is optional, and BigTIFF capture
metadata is not supported by the current EXIF reader.

## Detail cache

Import always decodes the source and saves the 640-pixel Gallery thumbnail,
but it does not render a detail image. Opening Detail for a HEIC or TIFF
shows the thumbnail first and requests a JPEG (up to 2,560 pixels on the
long edge). A cache hit uses the existing JPEG without decoding the source.
On a miss the app rebuilds it from the original in the background. Gallery
does not request detail JPEGs, and neither does navigating past a file
while another detail image is being prepared. JPEG, PNG, and WebP still
display the original directly in Detail.

The cache lives under `.photart/thumbnails/<shoot>/detail/`. It is not tracked
in the Catalog manifest. It survives app restarts and moves with the Shoot
on rename. It is safe to delete that `detail/` directory; photographs get
new detail JPEGs as they are opened again. No automatic eviction is applied.
Detail JPEGs from earlier imports at the same path can be reused. If the
detail image cannot be prepared, the thumbnail remains visible.

## Decoder comparison

| Decoder | Compatibility | Packaging and license |
| --- | --- | --- |
| [zune-heic 0.5](https://docs.rs/zune-heic/0.5.0/zune_heic/) (chosen) | 8-bit HEVC stills, grids, item transforms and EXIF support, but no 10-bit/4:2:2/4:4:4. | Rust software path on macOS, Windows and Linux; no external HEVC codec, DLL, or Homebrew package required. Photart disables optional VideoToolbox decoding so it exercises the same decoder on all three. MIT, Apache-2.0, or Zlib. |
| [libheif](https://github.com/strukturag/libheif) with [libde265](https://github.com/strukturag/libde265) | Broader HEIF/HEVC coverage, including Main 10 with libde265; other codecs depend on the configured plugins. | Native libraries/codecs to build or distribute for each macOS, Windows, and Linux target (CMake, vcpkg on Windows, or system packages during development). libheif is LGPL-3.0-or-later; libde265 is LGPL-3.0. The [Rust wrapper](https://github.com/Cykooz/libheif-rs) is MIT but does not change the native libraries' licenses. |
| [image](https://github.com/image-rs/image) and [image-tiff](https://github.com/image-rs/image-tiff) (chosen) | Reuses Photart's pixel and resize pipeline and exposes TIFF directory tags for RAW checks. More restricted than LibTIFF for some TIFF layouts and pixel encodings. | Pure Rust on macOS, Windows, and Linux, with no extra system TIFF dependency. `image` is MIT/Apache-2.0; `tiff` is MIT. |
| [LibTIFF](https://libtiff.gitlab.io/libtiff/) | Broader TIFF and BigTIFF coverage, with codec support depending on its build. | Portable native C library with a permissive [BSD-style license](https://libtiff.gitlab.io/libtiff/license.html). Would add a per-platform library and potentially other native codec dependencies. |

HEVC patent licensing is separate from these libraries' source licenses;
review distribution requirements before shipping the decoder. See the
[zune-heic patent notice](https://docs.rs/zune-heic/0.5.0/zune_heic/#patents).
TIFF/RAW identification is based in part on the [Adobe DNG specification](https://helpx.adobe.com/camera-raw/desktop/dng-and-file-formats/digital-negative.html).

## Candidates for later (not supported)

- **[AVIF](https://aomediacodec.github.io/av1-avif/)** — useful for modern photo
  exports. It needs an AV1 decoder, not the HEVC path used for HEIC. Color and
  HDR rendering would need their own acceptance criteria.
- **[JPEG XL](https://ds.jpeg.org/whitepapers/jpeg-xl-whitepaper.pdf)** — useful
  for archives, lossless images, and reversible JPEG recompression. Evaluate
  a dedicated decoder and reuse the JPEG-display-copy pipeline.
- **[BMP](https://github.com/image-rs/image)** — a lower-priority, small addition
  for legacy scans and older exports, using the existing Rust image library.
- **[GIF](https://github.com/image-rs/image)** — defer unless there is demand
  for static graphics; animation would require an explicit still-frame policy.

## Test fixture

`src-tauri/tests/fixtures/rotated.heic` is a synthetic 64 × 48 RGB test pattern
encoded with libheif as 8-bit HEVC 4:2:0, with a 90° clockwise item rotation
and a small EXIF block containing `Make=Photart` and `Orientation=6`. The item
rotation makes double-rotation detectable. Tests need no native encoder;
the TIFF, JPEG, PNG, and WebP test inputs are generated in Rust.
