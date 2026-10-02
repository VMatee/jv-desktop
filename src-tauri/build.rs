fn main() {
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        // The library's MockRuntime test executable also imports TaskDialogIndirect.
        // Activate Common Controls 6 there, not only in Tauri's application resource.
        println!("cargo:rustc-link-arg=/MANIFEST:EMBED");
        println!(
            "cargo:rustc-link-arg=/MANIFESTINPUT:{}",
            std::path::PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR").unwrap())
                .join("windows.manifest")
                .display()
        );
        println!("cargo:rerun-if-changed=windows.manifest");
    }
    // The linker embeds the common-controls manifest in application, DLL and
    // test targets, rather than a resource with one target-specific manifest ID.
    let attributes = tauri_build::Attributes::new()
        .windows_attributes(tauri_build::WindowsAttributes::new_without_app_manifest());
    tauri_build::try_build(
        attributes.app_manifest(tauri_build::AppManifest::new().commands(&[
            "choose_workspace",
            "detect_backend",
            "run_turn",
            "cancel_turn",
            "open_bridge",
            "close_bridge",
            "authenticate",
        ])),
    )
    .expect("Tauri build configuration could not be generated");
}
