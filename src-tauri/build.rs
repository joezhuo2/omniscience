fn main() {
    #[cfg(feature = "standalone")]
    tauri_build::build()
}
