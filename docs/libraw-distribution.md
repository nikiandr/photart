# LibRaw distribution for Photart

Scope: a paid or subscription-based Photart desktop app downloaded from our
website, starting with a macOS DMG. This documents the build and the remaining
publication checklist. It is not legal sign-off.

## Recommendation

Photart builds the unmodified **LibRaw 0.22.3** source archive under **CDDL
1.0**, checks its SHA-256 checksum and compiles it statically with the target's
C++ compiler. It does not use `rawlib`, system LibRaw, external codec packs,
or Homebrew. The archive and notices are bundled with the app under
`third-party/libraw` (under `Contents/Resources` on macOS).

LibRaw offers a choice of [CDDL 1.0 or LGPL 2.1][libraw-licensing].
[CDDL §§2.1 and 3.5–3.6][cddl] allow a royalty-free distribution of covered
code in a larger product. The [LibRaw maintainer also confirms][maintainer]
that dynamic linking isn't required under CDDL. The license applies to
LibRaw-covered files, including modifications to those files and new files
containing covered code, not to independent Photart code merely because it
links to LibRaw. Photart can be proprietary, sold as a download, or unlocked
by subscription. The subscription must not restrict recipients' CDDL rights
to the LibRaw source. Keep the adapter separate and do not copy LibRaw
implementation into it.

[LGPL 2.1 §6][lgpl] is an alternative, not an additional obligation when we
choose CDDL for the dual-licensed core. LGPL generally calls for a way to
replace or relink the library (for example, compatible shared-library
replacement or providing relinkable app object files and corresponding
library source). It also has notice and license-term requirements, including
permitting reverse engineering for debugging library modifications. A `.dylib`
alone does not establish that replacement works. CDDL has no equivalent
relinking requirement.

## Release checklist

1. **Build and inventory.** Pin the exact LibRaw source and record its build
   flags per target. Audit what goes into the actual artifact: LibRaw's
   [included code notices][copyright], the Rust wrapper if used, and any
   additional native dependencies such as JPEG, zlib, or Little CMS. Their
   terms are independent of the CDDL choice. Do not include separate GPL
   demosaic packs. Do not reuse the spike's Homebrew binary as a release input.
2. **Make matching source available (CDDL §§3.1–3.4).** Host a versioned archive
   of the LibRaw source used for each released binary, including covered
   changes, the CDDL license text, and intact upstream notices. Identify
   Photart's changes and contributor in modified covered files. Host the
   archive next to the installer without a subscription gate in addition to
   the copy in the app. Do not point only at upstream `main`.
3. **Tell recipients where to get source.** Put the LibRaw version, license
   choice, attribution, and matching source URL into Third-Party Notices
   shipped with the app. Also link the notices and archive on the download
   page. Have counsel review the EULA and paid-support terms for §§3.4–3.5,
   including the source-rights carve-out and the provisions on who offers
   additional terms and bears the related liability.
4. **Verify the customer build.** Check `otool -L` for unexpected development
   paths such as `/opt/homebrew`. Sign with Developer ID, notarize the release
   using [Tauri's macOS workflow][tauri-signing], and test the downloaded DMG on
   a clean Mac without Homebrew. Import the RAW sample folder and repeat on
   each architecture and platform we promise to support.

This covers the proposed LibRaw path, not a whole-app dependency, patent, or
subscription compliance audit. In particular, the existing
[HEIC/HEVC patent question](photo-import.md#decoder-comparison) is separate.
Have counsel review the final artifact, notices, source availability, and
customer terms before the first public release.

[libraw-licensing]: https://www.libraw.org/about#licensing
[cddl]: https://github.com/LibRaw/LibRaw/blob/0.22.3/LICENSE.CDDL
[maintainer]: https://www.libraw.org/node/2012
[lgpl]: https://github.com/LibRaw/LibRaw/blob/0.22.3/LICENSE.LGPL
[copyright]: https://github.com/LibRaw/LibRaw/blob/0.22.3/COPYRIGHT
[tauri-signing]: https://v2.tauri.app/distribute/sign/macos/
