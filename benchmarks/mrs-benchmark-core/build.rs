fn main() {
    println!("cargo:rerun-if-changed=../inputs.json");
    println!("cargo:rerun-if-env-changed=OUT_DIR");
    println!("cargo:rerun-if-env-changed=PROFILE");
    println!("cargo:rerun-if-env-changed=OPT_LEVEL");

    let out_dir = std::env::var("OUT_DIR").unwrap_or_default();
    let opt_level = std::env::var("OPT_LEVEL").unwrap_or_default();

    // Cargo routes intermediate artifacts to target/[<triple>/]<profile>/build/...
    let profile_tag = if out_dir.contains("/size/") || opt_level == "z" || opt_level == "s" {
        "size"
    } else if out_dir.contains("/lto/") {
        "lto"
    } else if out_dir.contains("/xlto/") {
        "xlto"
    } else if out_dir.contains("/release/") || opt_level == "3" {
        "release"
    } else {
        "dev"
    };

    println!("cargo:rustc-env=BENCH_PROFILE={profile_tag}");
}
