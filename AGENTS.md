# AGENTS.md

## Pre-push CI checks (required)
Always run these before pushing.

### Environment
```sh
export PURELY_STD_CRATES="ink/codegen metadata engine e2e e2e/macro ink/ir"
export ALSO_RISCV_CRATES="env storage storage/traits allocator prelude primitives ink ink/macro"
export CLIPPY_ALLOWED="clippy::extra_unused_lifetimes"
export RISCV_TARGET=".github/riscv64emac-unknown-none-polkavm.json"
export CLIPPY_TARGET="riscv64imac-unknown-none-elf"
```

### Prepare contract list (needed for examples checks)
```sh
scripts/for_all_contracts_exec.sh --output ./all-contracts --path . -- echo {}
```

### Spellcheck
```sh
cargo spellcheck check -v --cfg=.config/cargo_spellcheck.toml --checkers hunspell --code 1 -- recursive .
cargo spellcheck check -v --cfg=.config/cargo_spellcheck.toml --checkers hunspell --code 1 -- recursive ./integration-tests/*
```

### Formatting
```sh
zepter run check
cargo +nightly fmt --all -- --check
cargo +nightly fmt --all -- --check ./crates/ink/tests/ui/contract/{pass,fail}/*.rs
cargo +nightly fmt --all -- --check ./crates/ink/tests/ui/trait_def/{pass,fail}/*.rs
```

### Examples formatting
```sh
cat ./all-contracts | \
  grep integration-tests | \
  scripts/for_all_contracts_exec.sh -- cargo +nightly fmt --manifest-path {} -- --check
```

### Clippy (all crates, all targets/features)
```sh
ALL_CRATES="${PURELY_STD_CRATES} ${ALSO_RISCV_CRATES}"
for crate in ${ALL_CRATES}; do
  cargo +nightly clippy --all-targets --all-features --manifest-path ./crates/${crate}/Cargo.toml \
    -- -D warnings -A ${CLIPPY_ALLOWED};
done
```

### Clippy (RISC-V crates)
```sh
for crate in ${ALSO_RISCV_CRATES}; do
  cargo +nightly clippy --no-default-features --manifest-path ./crates/${crate}/Cargo.toml \
    --target ${CLIPPY_TARGET} \
    -- -D warnings -A ${CLIPPY_ALLOWED};
done
```

### Clippy (examples)
```sh
ALLOWED_LINTS="-A unexpected_cfgs"
while IFS= read -r contract; do
  cargo clippy --all-targets \
    --manifest-path $contract -- -D warnings -A $CLIPPY_ALLOWED $ALLOWED_LINTS;
done < <(grep integration-tests ./all-contracts)
```

```sh
ALLOWED_LINTS="-A unexpected_cfgs"
RUSTFLAGS="--cfg substrate_runtime"
while IFS= read -r contract; do
  cargo clippy --no-default-features \
    --target ${CLIPPY_TARGET} \
    --manifest-path ${contract} -- -D warnings -A $CLIPPY_ALLOWED $ALLOWED_LINTS;
done < <(grep integration-tests ./all-contracts)
```

### Check (all crates, std)
```sh
ALL_CRATES="${PURELY_STD_CRATES} ${ALSO_RISCV_CRATES}"
for crate in ${ALL_CRATES}; do
  cargo check --all-features --manifest-path ./crates/${crate}/Cargo.toml;
done
```

### Check (RISC-V no-default-features)
```sh
for crate in ${ALSO_RISCV_CRATES}; do
  cargo check --no-default-features --target $RISCV_TARGET -Zbuild-std="core,alloc" \
    --manifest-path ./crates/${crate}/Cargo.toml;
done
```

### Dylint checks
```sh
pushd linting
cargo check --verbose
cargo +nightly fmt --all -- --check
cargo clippy -- -D warnings
popd
```

### Dylint tests
```sh
pushd linting
cargo test --all-features
popd
```

## Scope of local checks
Do not try to run every CI workflow locally. The commands above are mandatory before pushing; run additional CI jobs locally only if your change touches them.
