fn main() {
    // Tell Cargo to rebuild this crate if inputs.json changes
    println!("cargo:rerun-if-changed=../inputs.json");
}
