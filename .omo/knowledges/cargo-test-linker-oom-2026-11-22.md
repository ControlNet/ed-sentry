# Memory-safe test execution (preventing OOM)

In environments with high baseline memory usage, high core counts (e.g. 16 vCPUs), and zero swap (`Swap: 0B`), running `cargo test --all` with default parallelism spawns up to `nproc` concurrent `rustc`/`lld` processes. Because `ed_sentry` links `tauri` (`wry`/`webkit2gtk`), `matrix-sdk`, `axum`, and `aws-lc-sys` across 27 top-level integration test targets (`tests/*.rs`), default `dev` debuginfo (`debug = 2`) plus parallel linking easily exceeds 8-20 GiB of free RAM and triggers OOM.

## Mandatory memory-safe test commands

1. **Always cap Cargo jobs (`-j 1` or `-j 2`) and disable DWARF debuginfo for test runs (`CARGO_PROFILE_DEV_DEBUG=0`)**:

```bash
CARGO_BUILD_JOBS=1 CARGO_PROFILE_DEV_DEBUG=0 cargo test --lib --bins -- --test-threads=2
```

2. **Run integration test targets individually or sequentially with `-j 1` and `CARGO_PROFILE_DEV_DEBUG=0`** rather than linking all 27 `tests/*.rs` binaries concurrently:

```bash
CARGO_BUILD_JOBS=1 CARGO_PROFILE_DEV_DEBUG=0 cargo test --test webui -- --test-threads=2
```

3. **Run Node and frontend checks sequentially (never in parallel with Cargo)**:

```bash
node --test scripts/*.test.mjs
pnpm --dir ui typecheck
pnpm --dir ui lint
pnpm --dir ui test
```

(`ui/vitest.config.ts` already enforces `maxThreads: 2` to cap Node worker memory.)
