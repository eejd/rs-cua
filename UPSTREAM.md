# Work adopted into `local`

*Provisional: the branch model below is planned while the uirunner full-workflow
tests are still being brought to green; review as needed.*

`main` mirrors `trycua/cua` `main` (fast-forward only, never edited). `local` is the integration
branch: upstream release tags are **merged** into it (`git merge --no-ff <release tag>`; never
rebased, never force-pushed once a tag exists), and each verified change branch (`feat/*`, `fix/*`)
listed below is merged into it with `--no-ff`. A new change branch is cut from an upstream release
tag, so it can become a selective upstream PR unchanged. Tags on `local`
are `<component prefix><base>-local<NN>` (the repo is a monorepo: `cua-driver-rs-v<base>-local<NN>`,
`lume-v<base>-local<NN>`; the `v<base>-local<NN>` shape of docs/development-model.md in
eejd/macports-ports-local gets the component prefix). From `cua-driver-rs-v0.38.0-local01` on, a
tag is immutable and stays reachable from `local`; earlier tags (`cua-driver-rs-v0.14.1-local1`,
`cua-driver-rs-v0.35.4-local01`) predate this rule, were cut when `local` was rewritten, and are
historical only. When upstream merges one of our changes, the next upstream merge brings it in and
the change branch is retired. Only an unpublished change branch (never merged into `local`, never
published as an upstream PR head) may be rebased onto a newer release tag.

**Pre-rule branches.** `fix/macos-sheet-key-scope` and `feat/macos-session-scope-click` were cut
from `local`, not from a release tag, and carry other adopted commits. They are not
upstream-PR-ready: re-cut each from the release tag (cherry-pick its own commit) before proposing
it upstream. Their tags (`-local03`, `-local04`) stay as they are.

| What | Branch | Source | Upstream status |
|---|---|---|---|
| macOS `AXIdentifier` exposed as structured element `id`; exact `id` in the `verify_state` selector (patchset `uirunner-macos-element-id-v1`) | `feat/macos-ax-identifier-id` | `eejd/swift-uirunner-broker@5e857a0010cd83fd9979d367e5e7a7815b9fe249` `packaging/cua-patches/uirunner-macos-element-id-v1.patch` (SHA-256 `cd1bbc28448e913d894af4eb26c4759d94d065bf53a5daf57d944420355983b2`; applies byte-for-byte to `cua-driver-rs-v0.38.0` with offsets only) | not proposed; mirrors the RFC trycua/cua#2798. No upstream PR until production soak. |
| macOS menu-opening `click`/`press` waits (bounded, single dispatch) for the menu to publish its items in the AX tree before returning (adopted with `eejd/rs-cua#11`; included in `cua-driver-rs-v0.38.0-local02`, cumulative patchset `uirunner-macos-recovery-v1`) | `feat/macos-menu-publication` | `eejd/rs-cua#11`, merged as `e952c2166649b190f39a83ce579dcc1e10df6057`; `eejd/swift-uirunner-broker` `packaging/cua-patches/uirunner-macos-recovery-v1.patch` carries the cumulative diff | not proposed. Live proof in #11 remains owed. |
| macOS `press_key` (plain Escape only) accepts a focused `AXSheet` attached to the addressed window as the key window, so a sheet can be dismissed with a background Escape (adopted with `eejd/rs-cua#13`; included in `cua-driver-rs-v0.38.0-local03`, cumulative patchset `uirunner-macos-recovery-v2`) | `fix/macos-sheet-key-scope` | `eejd/rs-cua#13`; live evidence: `window_not_key` refusal (focused window = the sheet) from the broker 0.2.17 resident | not proposed. |
| macOS `click` accepts `pointer_transport: "session"` (desktop scope): a screen-absolute click posted at the calling login session's event tap, with no cursor warp and no global modifier transitions, so a session that is not the console can click its own status item (adopted with `eejd/rs-cua#14`; included in `cua-driver-rs-v0.38.0-local04`, cumulative patchset `uirunner-macos-recovery-v3`) | `feat/macos-session-scope-click` | `eejd/rs-cua#14`; tracking `eejd/swift-uirunner-broker#180`; research: Chromium Remote Desktop and OSXvnc use the session tap off-console | not proposed. |
| Lume VNC framebuffer: `VNCClient` decodes little-endian XRGB pixels as actual RGB, rejects short buffers, and keeps requesting until real pixels arrive after DesktopSize-only updates (empty Raw rectangles do not count) (included in `lume-v0.7.2-local01`) | `fix/lume-vnc-framebuffer` | `eejd/rs-cua@a25a45481598535cfb22309d109ff3989b0030c5`, merged as `354ade5676538aae1021d20b36b2b03e626991a9` on top of the upstream `lume-v0.7.2` release (`f30fbc07a1d04879b5e4bce1e3cb10b12998bd96`), itself merged into `local` as `5e953673868659e183639894218536b82331902e` (`eejd/rs-cua#18`). The seven `VNCFramebufferTests` use scripted byte streams: unit tests, not live-guest proof. The same integration advanced `cua-driver` from 0.38.0 to 0.47.19 and carried the four driver patches above forward, with the Escape-only sheet allowance adapted to upstream `AxWindowId` (`own`/`root`), not applied byte-for-byte. `cd-swift-lume.yml` excludes `lume-v*-local*` tags from upstream's signing/publication pipeline | not proposed. |
| `cua-driver` advanced to upstream `cua-driver-rs-v0.55.3` (`8875d5199352f987d93c4879cf74974ff81fa554`), merged into `local` as `0bd061082183613b050d1390ca910e4a5def8c27` (`eejd/rs-cua#20`; included in `cua-driver-rs-v0.55.3-local01`). The four driver behaviors above were carried forward; they are no longer applied to base 0.38.0, so the `local01..04` tags and patchset ids `uirunner-macos-*` describe the old base. Adaptations: (1) the macOS menu-opening `click` wait is now **upstream's** `menu_beside` wait (upstream #8103, bounded, single dispatch, AXMenuButton/AXPopUpButton only) and the local duplicate in `click` is removed; `menu_state` keeps only the `set_value` read, and the driver no longer waits after `AXMenuBarItem` presses, which upstream deliberately excludes; (2) upstream moved display-only rows into `observations`, so `WindowObservation` gained the same optional `id` as `WindowElement` and `verify_state` still selects display-only text by exact `id`; (3) the Escape-only sheet scope composes with upstream's GdkQuartz activation lease in `key_scope_for`; (4) the session-tap click refuses on a security prompt like upstream's HID primitives (found in review of `eejd/rs-cua#19`). Unit and contract tests only; no live driver or GUI validation | `local` | `eejd/rs-cua#20`; upstream tag `cua-driver-rs-v0.55.3` | not proposed. |
