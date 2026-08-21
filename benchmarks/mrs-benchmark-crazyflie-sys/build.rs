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
    let lee_wrapper_c = manifest_dir.join("c_src/cf_lee_controller.c");
    let wrapper_h = manifest_dir.join("c_src/crazyflie_fw.h");

    // Tell Cargo to rebuild if the wrapper files or inputs change
    println!("cargo:rerun-if-changed={}", wrapper_c.display());
    println!("cargo:rerun-if-changed={}", lee_wrapper_c.display());
    println!("cargo:rerun-if-changed={}", wrapper_h.display());

    let c_src_dir = manifest_dir.join("c_src");
    // CMSIS-Core headers (cmsis_compiler.h etc.) still come from crazyflie-firmware's nested
    // CMSIS_5 checkout: CMSIS-DSP depends on them but they didn't move when DSP split out.
    let cmsis_core_dir = cf_src.join("../vendor/CMSIS/CMSIS");
    let cmsis_dsp_dir = manifest_dir.join("../vendor/CMSIS-DSP");

    let stub_autoconf = c_src_dir.join("stub_autoconf");
    let project_includes = [
        c_src_dir.clone(),
        stub_autoconf.clone(),
        cf_src.clone(),
        cf_src.join("modules/interface"),
        cf_src.join("modules/interface/kalman_core"),
        cf_src.join("modules/interface/controller"),
        cf_src.join("hal/interface"),
        cf_src.join("platform/interface"),
        cf_src.join("config"),
        cf_src.join("deck/interface"),
        cf_src.join("utils/interface"),
        cf_src.join("utils/interface/lighthouse"),
        cmsis_core_dir.join("Core/Include"),
        cmsis_dsp_dir.join("Include"),
        cmsis_dsp_dir.join("PrivateInclude"),
    ];

    let kalman_core_c = cf_src.join("modules/src/kalman_core/kalman_core.c");
    let lee_c = cf_src.join("modules/src/controller/controller_lee.c");
    let lee_wrapper_c = c_src_dir.join("cf_lee_controller.c");

    let cmsis_matrix_src = cmsis_dsp_dir.join("Source/MatrixFunctions");
    let cmsis_fastmath_src = cmsis_dsp_dir.join("Source/FastMathFunctions");
    let cmsis_basicmath_src = cmsis_dsp_dir.join("Source/BasicMathFunctions");
    let cmsis_tables_src = cmsis_dsp_dir.join("Source/CommonTables");

    let target = env::var("TARGET").unwrap();

    // 1. Compile the wrapper C code and CMSIS-DSP routines using the cc crate
    let mut cc_build = cc::Build::new();
    cc_build.file(&wrapper_c);
    cc_build.file(&lee_wrapper_c);
    cc_build.file(&lee_c);
    cc_build.file(&kalman_core_c);
    cc_build.file(cmsis_matrix_src.join("arm_mat_init_f32.c"));
    cc_build.file(cmsis_matrix_src.join("arm_mat_mult_f32.c"));
    cc_build.file(cmsis_matrix_src.join("arm_mat_add_f32.c"));
    cc_build.file(cmsis_matrix_src.join("arm_mat_sub_f32.c"));
    cc_build.file(cmsis_matrix_src.join("arm_mat_trans_f32.c"));
    cc_build.file(cmsis_matrix_src.join("arm_mat_inverse_f32.c"));
    cc_build.file(cmsis_basicmath_src.join("arm_dot_prod_f32.c"));
    cc_build.file(cmsis_fastmath_src.join("arm_cos_f32.c"));
    cc_build.file(cmsis_fastmath_src.join("arm_sin_f32.c"));
    cc_build.file(cmsis_tables_src.join("arm_common_tables.c"));

    if target.starts_with("thumbv7em") {
        cc_build.define("ARM_MATH_CM4", None);
    } else if target.starts_with("thumbv8m") {
        cc_build.define("ARM_MATH_CM33", None);
    } else if target.starts_with("thumbv6m") {
        cc_build.define("ARM_MATH_CM0PLUS", None);
    }
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

    // 3. Link the C compiler toolchain's native math library (libm.a / -lm).
    //
    // Without whole-archive, compiler_builtins's weak sqrtf/sinf/etc shadow the real libm.a
    // symbols (rustc links it before -lm, and archive scanning is lazy) -- see docs/platforms.md.
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

        let output = cmd
            .output()
            .expect("Failed to run arm-none-eabi-gcc -print-file-name=libm.a");
        assert!(output.status.success(), "arm-none-eabi-gcc failed");
        let path_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
        let path = PathBuf::from(&path_str);
        assert!(
            path.is_absolute() && path.exists(),
            "arm-none-eabi-gcc did not resolve a real libm.a (got {path_str:?})"
        );
        let dir = path.parent().expect("libm.a path has no parent dir");
        println!("cargo:rustc-link-search=native={}", dir.display());

        // rustc-link-arg=--whole-archive is a no-op here (only applies to this crate's own
        // artifact, an rlib); the +whole-archive link modifier on rustc-link-lib does propagate.
        // Skipped on thumbv6m (RP2040): fat LTO makes compiler_builtins's copy load-bearing
        // there too, so whole-archiving collides with it instead -- see docs/platforms.md.
        if target == "thumbv6m-none-eabi" {
            println!("cargo:rustc-link-lib=m");
        } else {
            println!("cargo:rustc-link-lib=static:+whole-archive=m");
        }
    } else {
        println!("cargo:rustc-link-lib=m");
    }
}
