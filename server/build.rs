fn main() {
    use std::env;

    // The directory containing the manifest for the package being built
    // (the package containing the build script).
    // Also note that this is the value of the
    // current working directory of the build script when it starts.
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|err| {
        panic!(
            "The CARGO_MANIFEST_DIR environment variable is not set: {}",
            err
        );
    });

    // Searching for Packet.lib (WinPcap) in possible existing folder ./lib
    println!("cargo:rustc-link-search=native={}/lib", manifest_dir);
}
