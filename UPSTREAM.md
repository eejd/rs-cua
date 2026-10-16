# Work adopted into `local`

`main` mirrors `trycua/cua` `main` (fast-forward only). `local` is a release-based
branch: the upstream tag it is cut from plus the work below. Tags on `local` are
`cua-driver-rs-v<base>-local<NN>` (the repo is a monorepo and cua-driver releases use
the component tag prefix `cua-driver-rs-v`; the `v<base>-local<NN>` shape of
docs/development-model.md in eejd/macports-ports-local gets this prefix).

| What | Source | Upstream status |
|---|---|---|
| macOS `AXIdentifier` exposed as structured element `id`; exact `id` in the `verify_state` selector (patchset `uirunner-macos-element-id-v1`) | `eejd/swift-uirunner-broker@5e857a0010cd83fd9979d367e5e7a7815b9fe249` `packaging/cua-patches/uirunner-macos-element-id-v1.patch` (SHA-256 `cd1bbc28448e913d894af4eb26c4759d94d065bf53a5daf57d944420355983b2`) | not proposed; mirrors the RFC trycua/cua#2798 (upstream submission is a gated step in the broker repo) |
