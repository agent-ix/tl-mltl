# V5 cargo-mutants process-group source

`cargo-mutants-27.0.0-quoin-process-group.patch` is the candidate patch for
the isolated Linux V5 diagnostic build. Apply it to the published
`cargo-mutants` 27.0.0 crate whose `.crate` SHA-256 is
`98d49ad7eef8a741a64593685fd0f49075ec562f6b4cb7432a3abaa707198ed5`.
The patch SHA-256 is
`39d735e51f23b122715fe03d7a75425157d80435c358143a3a9de503454e6251`.
It changes `src/process/unix.rs` and `src/copy_tree.rs`; the default private child process
group remains the default. With the exact environment value
`TL_QUOIN_PROCESS_GROUP_V1=1`, a child inherits Quoin's invocation group,
and cargo-mutants sends TERM then bounded KILL to its direct child rather
than signalling Quoin's whole group. Descendants are left for Quoin's outer
group cleanup. The upstream `src/process.rs` general comment describes the
default mode; this opt-in is its explicit exception.
With the same exact opt-in, regular files in cargo-mutants' **private**
source copy gain owner-write permission after copy. Quoin's sealed 0400
source remains read-only. This permits mutation writes without changing the
sealed projection or default cargo-mutants behavior.

For an isolated build, extract the `.crate` archive, enter its
`cargo-mutants-27.0.0` directory, apply the patch with `patch -p1`, then run
the three `process::unix::tests` and the private-copy test, then build
`cargo-mutants` from that source.
Do not replace a shared cargo-mutants installation. The earlier
process-group-only patch had SHA-256
`e33085b90eb730fdf23b58e0799e838460525c13b1140afff8bcbdddb4f5bea`.
The cave diagnostic generated `Cargo.lock` SHA-256 for that earlier patch
`fb9b62d084e14b90ca3c15180aa4a858a302db9dbc01e5b9f654593f7d3114f5`,
patched Unix source SHA-256
`baefa7a5308f7b6ad9f774f55ac6a265ce87da1053492c687e3055cc006e01e5`,
and Linux executable SHA-256
`cd71b6bfaef10dcfa99fafa3d3ad6b9b1f0b4760cfd947c4e69d029a81c2c76e`.
All three focused process tests passed on that earlier patch; their log SHA-256 is
`c6bbdbcd33a4a9899054b7ee27e3e4c432dfc4699e8141b4f953b12f7010b7e3`.
The cave build manifest is
`/tmp/tl246-cave-20260928/diagnostics/v5-pg1-build/build-manifest.json`
(SHA-256 `015c1fbf6a7c486b9204d2f6fbe782ff44223f77a358a62c4af6966746c5edd5`).

The earlier selected V5 mutation attempt reached a native `Permission denied`
writing its private copied `src/infinite/mod.rs` and correctly rejected after
discovering 44 mutants. That refusal motivates the new `copy_tree.rs` change.
The new patch has passed local application dry-run and rustfmt; its Linux
build, focused tests, selected mutation run, and independent replay remain
pending. Neither the earlier process tests nor selected discovery establish
Quoin's outer cleanup, mutation population, V5 acceptance, or Campaign
acceptance. Those require selected sealed procedure runs and independent
receipt replay on a source graph that pins the updated patch and executable.
