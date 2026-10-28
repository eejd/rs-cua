# Work adopted into `local`

*Provisional: the branch model below is planned while the uirunner full-workflow
tests are still being brought to green; review as needed.*

`main` mirrors `trycua/cua` `main` (fast-forward only). `local` is rebuilt on each
upstream release: the release tag it is cut from, plus a `--no-ff` merge of each
verified `feat/*` branch listed below. Each `feat/*` branch is itself cut from the
same release tag, so it can later become an upstream PR unchanged. Tags on `local`
are `cua-driver-rs-v<base>-local<NN>` (the repo is a monorepo and cua-driver releases
use the component tag prefix `cua-driver-rs-v`; the `v<base>-local<NN>` shape of
docs/development-model.md in eejd/macports-ports-local gets this prefix). A tag is
immutable; `local` itself is rewritten when it is rebuilt on a newer release.

| What | Branch | Source | Upstream status |
|---|---|---|---|
| macOS `AXIdentifier` exposed as structured element `id`; exact `id` in the `verify_state` selector (patchset `uirunner-macos-element-id-v1`) | `feat/macos-ax-identifier-id` | `eejd/swift-uirunner-broker@5e857a0010cd83fd9979d367e5e7a7815b9fe249` `packaging/cua-patches/uirunner-macos-element-id-v1.patch` (SHA-256 `cd1bbc28448e913d894af4eb26c4759d94d065bf53a5daf57d944420355983b2`; applies byte-for-byte to `cua-driver-rs-v0.38.0` with offsets only) | not proposed; mirrors the RFC trycua/cua#2798. No upstream PR until production soak. |
| macOS menu-opening `click`/`press` waits (bounded, single dispatch) for the menu to publish its items in the AX tree before returning (adopted with `eejd/rs-cua#11`; included in `cua-driver-rs-v0.38.0-local02`, cumulative patchset `uirunner-macos-recovery-v1`) | `verify/macos-menu-publication` | `eejd/rs-cua#11`, merged as `e952c2166649b190f39a83ce579dcc1e10df6057`; `eejd/swift-uirunner-broker` `packaging/cua-patches/uirunner-macos-recovery-v1.patch` carries the cumulative diff | not proposed. Live proof in #11 remains owed. |
| macOS `press_key` (plain Escape only) accepts a focused `AXSheet` attached to the addressed window as the key window, so a sheet can be dismissed with a background Escape (adopted with `eejd/rs-cua#13`; included in `cua-driver-rs-v0.38.0-local03`, cumulative patchset `uirunner-macos-recovery-v2`) | `fix/macos-sheet-key-scope` | `eejd/rs-cua#13`; live evidence: `window_not_key` refusal (focused window = the sheet) from the broker 0.2.17 resident | not proposed. |
