// Writes the icon files the installers need (target/icons) and, on Windows,
// embeds the icon and version info into ccdc.exe.
#[path = "src/icon.rs"]
mod icon;

use std::path::PathBuf;

fn main() {
    println!("cargo:rerun-if-changed=src/icon.rs");
    println!("cargo:rerun-if-changed=build.rs");
    let root = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    let dir = root.join("target").join("icons");
    std::fs::create_dir_all(&dir).unwrap();
    for s in [16u32, 32, 64, 128, 256, 512, 1024] {
        let png = icon::png(s, &icon::draw(s, icon::ORANGE));
        std::fs::write(dir.join(format!("ccdc-{s}.png")), png).unwrap();
    }
    let ico_path = dir.join("ccdc.ico");
    std::fs::write(&ico_path, icon::ico(&[16, 24, 32, 48, 64, 256], icon::ORANGE)).unwrap();

    // this platform's version, from [package.metadata.ccdc] in Cargo.toml
    println!("cargo:rerun-if-changed=Cargo.toml");
    let os = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    let key = match os.as_str() {
        "windows" => "windows",
        "macos" => "macos",
        _ => "linux",
    };
    let manifest = std::fs::read_to_string(root.join("Cargo.toml")).unwrap();
    let version = manifest
        .split("[package.metadata.ccdc]")
        .nth(1)
        .and_then(|sec| sec.lines().find_map(|l| l.trim().strip_prefix(key)?.trim().strip_prefix('=')?.trim().strip_prefix('"')?.split('"').next()))
        .unwrap_or_else(|| panic!("no {key} version in [package.metadata.ccdc]"))
        .to_string();
    println!("cargo:rustc-env=CCDC_VERSION={version}");

    if os == "windows" {
        let mut res = winresource::WindowsResource::new();
        res.set_icon(ico_path.to_str().unwrap());
        res.set("ProductVersion", &version);
        res.set("FileVersion", &version);
        res.set("ProductName", "Continuous Claude Desktop Code");
        res.set("FileDescription", "CCDC (Continuous Claude Desktop Code)");
        res.set("CompanyName", "CCDC");
        res.set("LegalCopyright", "MIT License");
        res.compile().expect("embedding the Windows icon");
    }
}
