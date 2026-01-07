use std::env;

fn main() {
    let inc = env::var("CHAKRACORE_INCLUDE_DIR")
        .expect("CHAKRACORE_INCLUDE_DIR not set (path to ChakraCore headers directory)");
    let lib = env::var("CHAKRACORE_LIB_DIR")
        .expect("CHAKRACORE_LIB_DIR not set (path to ChakraCore import library directory)");

    println!("cargo:rerun-if-env-changed=CHAKRACORE_INCLUDE_DIR");
    println!("cargo:rerun-if-env-changed=CHAKRACORE_LIB_DIR");

    // Expose include dir as build metadata (useful for downstream tooling)
    println!("cargo:include={}", inc);

    println!("cargo:rustc-link-search=native={}", lib);

    // Common Windows import library name: ChakraCore.lib
    // If your import library name differs, adjust this line.
    println!("cargo:rustc-link-lib=dylib=ChakraCore");
}
