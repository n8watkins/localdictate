fn main() {
    // The real Google OAuth client id/secret live in the gitignored
    // src/google_secrets.rs. Recreate it from the committed template on a
    // fresh clone so the crate always compiles (with "not configured"
    // placeholders until real values are filled in).
    let secrets = std::path::Path::new("src/google_secrets.rs");
    if !secrets.exists() {
        if let Err(error) = std::fs::copy("src/google_secrets.example.rs", secrets) {
            println!("cargo:warning=Could not create src/google_secrets.rs: {error}");
        }
    }
    tauri_build::build()
}
