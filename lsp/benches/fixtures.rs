// Shared deterministic fixture generator for LSP benches.
//
// Each bench file `include!`s this file so we don't ship a "common"
// crate for four benches. The generator emits realistic Ktav content:
// a mix of plain pairs, dotted keys, typed scalars (`:i`, `:f`), raw
// markers (`::`), nested objects/arrays, and a few comment lines.
//
// Sizes are TARGET byte counts. The function appends pairs until the
// buffer reaches the target, so the actual size will be slightly above.

#[allow(dead_code)]
pub fn small() -> String {
    synth(1_024)
}

#[allow(dead_code)]
pub fn medium() -> String {
    synth(50 * 1_024)
}

#[allow(dead_code)]
pub fn large() -> String {
    synth(500 * 1_024)
}

/// Synthesize a Ktav document at least `target_bytes` long.
#[allow(dead_code)]
pub fn synth(target_bytes: usize) -> String {
    let mut out = String::with_capacity(target_bytes + 256);
    out.push_str("# generated benchmark fixture\n");
    let mut i = 0u32;
    // Round-robin pattern keeps the mix realistic without exploding
    // file structure (deeply-nested generated content tickles parser
    // edge cases, not steady-state perf).
    while out.len() < target_bytes {
        match i % 12 {
            0 => {
                use std::fmt::Write as _;
                let _ = writeln!(out, "name_{}: value_{}", i, i);
            }
            1 => {
                use std::fmt::Write as _;
                let _ = writeln!(out, "port_{}:i {}", i, 8000 + (i as u64 % 1000));
            }
            2 => {
                use std::fmt::Write as _;
                let _ = writeln!(out, "ratio_{}:f {}.{}", i, i % 100, i % 1000);
            }
            3 => {
                use std::fmt::Write as _;
                let _ = writeln!(
                    out,
                    "flag_{}: {}",
                    i,
                    if i % 2 == 0 { "true" } else { "false" }
                );
            }
            4 => {
                use std::fmt::Write as _;
                let _ = writeln!(
                    out,
                    "label_{}:: literal text with spaces and symbols !@#${}",
                    i, i
                );
            }
            5 => {
                use std::fmt::Write as _;
                let _ = writeln!(out, "service.{}.host: 10.0.0.{}", i, i % 256);
            }
            6 => {
                use std::fmt::Write as _;
                let _ = writeln!(out, "service.{}.port:i {}", i, 30000 + (i as u64 % 5000));
            }
            7 => {
                use std::fmt::Write as _;
                let _ = writeln!(out, "# section {}", i / 12);
            }
            8 => {
                use std::fmt::Write as _;
                let _ = writeln!(out, "obj_{}: {{", i);
                let _ = writeln!(out, "    inner_a: {}", i);
                let _ = writeln!(out, "    inner_b:f {}.5", i % 100);
                let _ = writeln!(out, "    inner_c:: raw body for {}", i);
                let _ = writeln!(out, "}}");
            }
            9 => {
                use std::fmt::Write as _;
                let _ = writeln!(out, "list_{}: [", i);
                let _ = writeln!(out, "    item-a-{}", i);
                let _ = writeln!(out, "    item-b-{}", i);
                let _ = writeln!(out, "    :: literal-{}", i);
                let _ = writeln!(out, "]");
            }
            10 => {
                use std::fmt::Write as _;
                // multi-line raw block
                let _ = writeln!(out, "doc_{}: (", i);
                let _ = writeln!(out, "first line of body {}", i);
                let _ = writeln!(out, "second line of body {}", i);
                let _ = writeln!(out, ")");
            }
            _ => {
                use std::fmt::Write as _;
                let _ = writeln!(out, "tag_{}: alpha-beta-gamma-{}", i, i);
            }
        }
        i = i.wrapping_add(1);
    }
    out
}
