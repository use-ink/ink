# AGENTS.md

## Pre-push CI sanity checks (required)
Run these before pushing when touching revive/runtime/env or CI-sensitive changes.

### Clippy (all crates, all targets/features)
```sh
for crate in ${ALSO_RISCV_CRATES}; do
  cargo +nightly clippy --all-targets --all-features --manifest-path ./crates/${crate}/Cargo.toml \
    -- -D warnings -A ${CLIPPY_ALLOWED};
done
```

### RISC-V no-default-features check
```sh
for crate in ${ALSO_RISCV_CRATES}; do
  cargo check --no-default-features --target $RISCV_TARGET -Zbuild-std="core,alloc" \
    --manifest-path ./crates/${crate}/Cargo.toml;
done
```

## Scope of local checks
Do not try to run every CI workflow locally. Only run targeted checks relevant to the change or the commands above.
