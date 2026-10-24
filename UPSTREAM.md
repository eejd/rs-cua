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
