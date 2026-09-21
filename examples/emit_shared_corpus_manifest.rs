//! Emit the compiled tl-syntax dependency's `corpus/manifest.json` bytes
//! verbatim (FR-006-AC-2).
//!
//! `scripts/assurance_chain.py` reads the shared corpus's declared malformed
//! count as an independent oracle for `reference_conformance`'s own output —
//! independent means it must not come from that same producer, or a producer
//! that stopped reporting malformed rows could also move the number it is
//! checked against. The chain driver is deliberately forbidden from running a
//! producer itself (its own `sys.addaudithook` refuses any child process that
//! is not the pinned Quoin CLI or a version observation), and the manifest
//! now lives inside the compiled dependency's checkout rather than a
//! vendored copy in this repository, so it cannot read the bytes directly
//! either. This producer is the bridge: it runs under `make assurance-inputs`
//! like every other producer and writes the manifest bytes it read, unchanged,
//! to a file the driver is allowed to read.

use std::fs;
use std::io::Write;
use std::path::Path;
use std::process::ExitCode;

fn main() -> ExitCode {
    let path = Path::new(tl_syntax::CORPUS_DIR).join("manifest.json");
    let bytes = match fs::read(&path) {
        Ok(bytes) => bytes,
        Err(error) => {
            eprintln!("read {}: {error}", path.display());
            return ExitCode::FAILURE;
        }
    };
    match std::io::stdout().write_all(&bytes) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("write stdout: {error}");
            ExitCode::FAILURE
        }
    }
}
