fn main() {
    use std::env;
    use std::path::Path;

    // The directory containing the manifest for the package being built
    // (the package containing the build script).
    // Also note that this is the value of the
    // current working directory of the build script when it starts.
    let manifest_dir = match env::var("CARGO_MANIFEST_DIR") {
        Ok(value) => value,
        Err(err) => {
            eprintln!("The CARGO_MANIFEST_DIR environment variable is not set: {err}");
            std::process::exit(1);
        },
    };

    let workspace_root = match Path::new(&manifest_dir).parent() {
        Some(value) => value,
        None => {
            eprintln!("Failed to get parent directory");
            std::process::exit(1);
        },
    };

    // Lib Folder
    let lib_path = workspace_root.join("lib");

    // Searching for Packet.lib (WinPcap) in possible existing folder ./lib
    println!("cargo:rustc-link-search=native={}", lib_path.display());
}
