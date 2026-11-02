use std::{env, fs, io::Read, path::PathBuf};

fn main() {
    println!("cargo:rerun-if-env-changed=KTAV_NATIVE_NOTICES");
    println!("cargo:rerun-if-changed=build.rs");
    let out = PathBuf::from(env::var_os("OUT_DIR").expect("Cargo must provide OUT_DIR"));
    let generated = out.join("native_notices.rs");
    let Some(path) = env::var_os("KTAV_NATIVE_NOTICES") else {
        fs::write(generated, "static KTAV_NATIVE_NOTICES: &[u8] = &[];\n")
            .expect("write development notice stub");
        return;
    };
    let path = PathBuf::from(path);
    println!("cargo:rerun-if-changed={}", path.display());
    let mut payload = Vec::new();
    fs::File::open(path)
        .expect("open prepared LSP notice gzip")
        .take(4 * 1024 * 1024 + 1)
        .read_to_end(&mut payload)
        .expect("read prepared LSP notice gzip");
    assert!(
        payload.starts_with(&[0x1f, 0x8b]) && payload.len() <= 4 * 1024 * 1024,
        "LSP notice payload must be a bounded gzip"
    );
    let length = u64::try_from(payload.len()).expect("LSP notice length must fit u64");
    let mut resource = b"KTAV-NOTICES-GZIP-v1\0".to_vec();
    resource.extend_from_slice(&length.to_le_bytes());
    resource.extend_from_slice(&payload);
    fs::write(out.join("native_notices.bin"), resource).expect("write LSP notice resource");
    fs::write(
        generated,
        r#"#[used]
static KTAV_NATIVE_NOTICES: &[u8] =
    include_bytes!(concat!(env!("OUT_DIR"), "/native_notices.bin"));
"#,
    )
    .expect("write LSP notice declaration");
}
