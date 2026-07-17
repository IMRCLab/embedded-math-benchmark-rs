use std::env;
use std::path::PathBuf;

// Workaround, not a clean fix: Debian's gcc-arm-none-eabi doesn't self-report a sysroot
// (Arch's does), so we make gcc resolve the wrapper header via `-M` and read off that instead.
fn gcc_system_include_dirs(
    wrapper_h: &std::path::Path,
    project_includes: &[PathBuf],
) -> Vec<String> {
    let mut cmd = cc::Build::new().get_compiler().to_command();
    cmd.arg("-M");
    for dir in project_includes {
        cmd.arg(format!("-I{}", dir.display()));
    }
    cmd.arg(wrapper_h);

    let output = cmd
        .output()
        .expect("Failed to run the C compiler to resolve the wrapper header's dependencies");
    assert!(
        output.status.success(),
        "gcc -M failed to resolve {}: {}",
        wrapper_h.display(),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);

    let mut dirs: Vec<String> = stdout
        .split_whitespace()
        .filter(|tok| tok.starts_with('/')) // drop the "target:" rule name and line continuations
        .filter_map(|path| Some(PathBuf::from(path).parent()?.to_str()?.to_string()))
        .collect();
    dirs.sort();
    dirs.dedup();
    dirs
}

fn main() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let cf_src = manifest_dir.join("../vendor/crazyflie-firmware/src");
    let wrapper_c = manifest_dir.join("c_src/cf_math_wrapper.c");
    let wrapper_h = manifest_dir.join("c_src/crazyflie_fw.h");

    // Tell Cargo to rebuild if the wrapper files or inputs change
    println!("cargo:rerun-if-changed={}", wrapper_c.display());
    println!("cargo:rerun-if-changed={}", wrapper_h.display());

    let project_includes = [
        cf_src.clone(),
        cf_src.join("modules/interface"),
        cf_src.join("hal/interface"),
        cf_src.join("utils/interface"),
        cf_src.join("utils/interface/lighthouse"),
    ];

    // 1. Compile the wrapper C code using the cc crate
    let mut cc_build = cc::Build::new();
    cc_build.file(&wrapper_c);
    for dir in &project_includes {
        cc_build.include(dir);
    }
    cc_build
        .flag_if_supported("-Wno-absolute-value")
        .flag_if_supported("-Wno-strict-aliasing")
        .compile("cf_math_wrapper");

    // 2. Generate the FFI bindings using bindgen
    let target = env::var("TARGET").unwrap();
    let mut builder = bindgen::Builder::default().header(wrapper_h.to_str().unwrap());
    for dir in &project_includes {
        builder = builder.clang_arg(format!("-I{}", dir.display()));
    }
    builder = builder
        .clang_arg("-Wno-absolute-value")
        .clang_arg("-Wno-strict-aliasing")
        .blocklist_item("FP_NAN")
        .blocklist_item("FP_INFINITE")
        .blocklist_item("FP_ZERO")
        .blocklist_item("FP_SUBNORMAL")
        .blocklist_item("FP_NORMAL")
        .use_core() // Ensure no_std output
        .ctypes_prefix("core::ffi"); // Use core::ffi types

    builder = builder.clang_arg(format!("--target={}", target));

    if target.starts_with("thumb") {
        for dir in gcc_system_include_dirs(&wrapper_h, &project_includes) {
            builder = builder.clang_arg(format!("-I{}", dir));
        }
    }

    builder = builder.layout_tests(false);

    let bindings = builder.generate().expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");

    // 3. Link the C compiler toolchain's native math library (libm.a / -lm)
    if target.starts_with("thumb") {
        let mut cmd = std::process::Command::new("arm-none-eabi-gcc");
        cmd.arg("-print-file-name=libm.a");

        match target.as_str() {
            "thumbv7em-none-eabihf" => {
                cmd.args([
                    "-mthumb",
                    "-march=armv7e-m",
                    "-mfloat-abi=hard",
                    "-mfpu=fpv4-sp-d16",
                ]);
            }
            "thumbv8m.main-none-eabihf" => {
                cmd.args(["-mthumb", "-march=armv8-m.main+fp", "-mfloat-abi=hard"]);
            }
            "thumbv6m-none-eabi" => {
                cmd.args(["-mthumb", "-march=armv6-m", "-mfloat-abi=soft"]);
            }
            _ => {}
        }

        if let Ok(output) = cmd.output() {
            if output.status.success() {
                let path_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
                let path = PathBuf::from(path_str);
                if let Some(dir) = path.parent() {
                    println!("cargo:rustc-link-search=native={}", dir.display());
                }
            }
        }
    }

    println!("cargo:rustc-link-lib=m");
}
