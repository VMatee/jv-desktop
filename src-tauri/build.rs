fn main() {
    tauri_build::try_build(tauri_build::Attributes::new().app_manifest(
        tauri_build::AppManifest::new().commands(&[
            "choose_workspace",
            "detect_backend",
            "run_turn",
            "cancel_turn",
        ]),
    ))
    .expect("Tauri build configuration could not be generated");
}
