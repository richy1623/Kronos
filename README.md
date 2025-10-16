# Kronos

[![codecov](https://codecov.io/gh/richy1623/Kronos/branch/tokio/graph/badge.svg?token=XIPY37MUZT)](https://codecov.io/gh/richy1623/Kronos)

<!-- TODO remove the tokio branch -->

## How to run tests with coverage

### Locally

```shell
cargo tarpaulin --no-fail-fast --target-dir "target/tarpaulin" --exclude-files "target/*" --skip-clean --out html
```

### On CI

Make a commit with a commit message containing `[ci]`
