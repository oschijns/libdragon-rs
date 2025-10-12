use std::{
    env, io,
    path::PathBuf,
    process::{exit, Command},
};

#[derive(Debug)]
struct Cb;

impl bindgen::callbacks::ParseCallbacks for Cb {
    fn process_comment(&self, comment: &str) -> Option<String> {
        //eprintln!("cmt: {:?}", comment);
        // Try to transform the comment, but fall back to original on error
        match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            doxygen_rs::transform(comment)
        })) {
            Ok(transformed) => Some(transformed),
            Err(_) => {
                // If doxygen transformation fails, just skip the transformation
                // and return the original comment
                eprintln!("cargo:warning=Failed to transform doxygen comment, using original");
                Some(comment.to_string())
            }
        }
    }
}

fn main() -> io::Result<()> {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let src_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let libdragon_dir = src_dir.clone().join("libdragon");
    let toolchain_dir = out_dir.clone().join("toolchain");

    // Check if N64_INST is set, if not use our out_dir toolchain
    let n64_inst =
        env::var("N64_INST").unwrap_or_else(|_| toolchain_dir.to_string_lossy().to_string());
    let n64_inst_path = PathBuf::from(&n64_inst);

    // Detect GCC version dynamically
    let gcc_version = {
        let gcc_dir = n64_inst_path.join("lib").join("gcc").join("mips64-elf");
        if gcc_dir.exists() {
            std::fs::read_dir(&gcc_dir)
                .ok()
                .and_then(|mut entries| entries.next())
                .and_then(|entry| entry.ok())
                .and_then(|entry| entry.file_name().into_string().ok())
                .unwrap_or_else(|| "14.2.0".to_string())
        } else {
            "14.2.0".to_string()
        }
    };

    // Create the build directory
    let libdragon_build_dir = out_dir.clone().join("libdragon_build");
    let mut mkdir = Command::new("mkdir");
    mkdir.arg("-p").arg(&libdragon_build_dir);
    let _ = mkdir.output()?;

    // Check if the toolchain exists
    let toolchain_bin = n64_inst_path.join("bin").join("mips64-elf-gcc");
    if !toolchain_bin.exists() {
        eprintln!("N64 toolchain not found at: {}", n64_inst_path.display());
        eprintln!("Please install the libdragon toolchain and set N64_INST environment variable.");
        eprintln!("See: https://github.com/DragonMinded/libdragon/wiki/Installing-libdragon");
        exit(1);
    }

    println!(
        "cargo:warning=Using N64 toolchain at: {}",
        n64_inst_path.display()
    );

    // build libdragon
    let mut make = Command::new("make");
    make.arg("-C")
        .arg(libdragon_dir.clone().into_os_string())
        .current_dir(&libdragon_build_dir)
        .arg("libdragon")
        .arg("tools")
        .arg("-j")
        .arg("4")
        .env("N64_INST", &n64_inst_path);

    let build_output = make.output();
    if build_output.is_err() || !build_output.as_ref().unwrap().status.success() {
        eprintln!("There was an error building libdragon");
        if let Ok(output) = build_output {
            eprintln!("Build stderr: {}", String::from_utf8_lossy(&output.stderr));
        }
        exit(1);
    }

    // Create a local install directory for libdragon
    let libdragon_install_dir = out_dir.join("libdragon_install");
    std::fs::create_dir_all(&libdragon_install_dir)?;

    // install libdragon and tools to local directory
    let mut install = Command::new("make");
    install
        .arg("-C")
        .arg(libdragon_dir.clone().into_os_string())
        .current_dir(&libdragon_build_dir)
        .arg("install")
        .arg("tools-install")
        .arg(format!("INSTALLDIR={}", libdragon_install_dir.display()))
        .env("N64_INST", &n64_inst_path);

    let install_output = install.output();
    if install_output.is_err() || !install_output.as_ref().unwrap().status.success() {
        eprintln!("There was an error installing libdragon");
        if let Ok(output) = install_output {
            eprintln!(
                "Install stderr: {}",
                String::from_utf8_lossy(&output.stderr)
            );
            eprintln!(
                "Install stdout: {}",
                String::from_utf8_lossy(&output.stdout)
            );
        }
        exit(1);
    }

    // Check that the libdragon libraries were installed
    let libdragon_lib = libdragon_install_dir
        .join("mips64-elf")
        .join("lib")
        .join("libdragon.a");
    let dragonsys_lib = libdragon_install_dir
        .join("mips64-elf")
        .join("lib")
        .join("libdragonsys.a");
    if !libdragon_lib.exists() || !dragonsys_lib.exists() {
        eprintln!("libdragon installation failed: libraries not found");
        eprintln!("Expected: {}", libdragon_lib.display());
        eprintln!("Expected: {}", dragonsys_lib.display());
        exit(1);
    }
    println!("cargo:warning=libdragon libraries installed successfully");

    // link against libdragon.a and libdragonsys.a from local install and toolchain
    println!(
        "cargo:rustc-link-search=native={}/mips64-elf/lib",
        libdragon_install_dir.display()
    );
    println!(
        "cargo:rustc-link-search=native={}/mips64-elf/lib",
        n64_inst_path.display()
    );
    println!("cargo:rustc-link-lib=static=dragon");
    println!("cargo:rustc-link-lib=static=dragonsys");

    println!(
        "cargo:rustc-link-search=native={}/lib/gcc/mips64-elf/{}",
        n64_inst_path.display(),
        gcc_version
    );
    println!("cargo:rustc-link-lib=static=c");
    println!("cargo:rustc-link-lib=static=g");
    println!("cargo:rustc-link-lib=static=nosys");
    println!("cargo:rustc-link-lib=static=gcc");
    println!("cargo:rustc-link-lib=static=m");

    // Verify that the toolchain was installed correctly
    let toolchain_include = n64_inst_path.join("mips64-elf").join("include");
    let errno_header = toolchain_include.join("sys").join("errno.h");
    if !errno_header.exists() {
        eprintln!(
            "Toolchain installation failed: {} does not exist",
            errno_header.display()
        );
        eprintln!(
            "Make sure libdragon toolchain is properly installed at N64_INST={}",
            n64_inst_path.display()
        );
        exit(1);
    }

    let static_fns_path = out_dir.clone().join("static_fns.c");

    let bindings = bindgen::Builder::default()
        .clang_arg(format!("-I{}/include", libdragon_dir.display()))
        .clang_arg(format!("-I{}/include", libdragon_install_dir.display()))
        .clang_arg(format!("-I{}/mips64-elf/include", n64_inst_path.display()))
        .clang_arg(format!("--sysroot={}/mips64-elf", n64_inst_path.display()))
        .clang_args(&[
            "-target",
            "mips-nintendo64-none",
            "-mabi=n32",
            "-DN64",
            "-isystem",
            &format!("{}/include", libdragon_install_dir.display()),
            "-isystem",
            &format!("{}/mips64-elf/include", n64_inst_path.display()),
            "-isystem",
            &format!(
                "{}/lib/gcc/mips64-elf/{}/include",
                n64_inst_path.display(),
                gcc_version
            ),
        ])
        .header("wrapper.h")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .parse_callbacks(Box::new(Cb {}))
        .use_core()
        .generate_inline_functions(true)
        .wrap_static_fns_path(&static_fns_path)
        .wrap_static_fns(true)
        .generate()
        .expect("Unable to generate a binding");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");

    // Compile the static_fns file
    // The compile arguments are taken from n64.mk, so they need to be kept in sync.
    let static_fns_obj_path = out_dir.clone().join("static_fns.o");
    let mut compile_fns = Command::new("clang");
    compile_fns
        .arg("-target")
        .arg("mips-nintendo64-none")
        .arg("-mabi=n32")
        .arg(format!("-ffile-prefix-map={}=", out_dir.clone().display()))
        .arg("-DN64")
        .arg("-Wall")
        .arg("-std=gnu99")
        .arg("-O2")
        //.arg("-flto=thin") // TODO: thin LTO
        .arg("-c")
        .arg("-o")
        .arg(&static_fns_obj_path)
        .arg(static_fns_path)
        .arg("-I")
        .arg(src_dir.clone())
        .arg("-I")
        .arg(n64_inst_path.clone().join("include"))
        .arg("-I")
        .arg(n64_inst_path.clone().join("mips64-elf").join("include"));

    eprintln!("compile: {:?}", compile_fns);
    if compile_fns.output().is_err() {
        eprintln!("Could not compile static_fns.c");
        exit(1);
    }

    // Add the static_fns.o object to an archive
    let mut add_archive = Command::new(n64_inst_path.clone().join("bin").join("mips64-elf-ar"));
    add_archive
        .arg("-crus")
        .arg(
            n64_inst_path
                .clone()
                .join("mips64-elf")
                .join("lib")
                .join("libextern.a"),
        )
        .arg(static_fns_obj_path);
    if add_archive.output().is_err() {
        eprintln!("Could not add static_fns.o to libdragon.a");
        exit(1);
    }

    Ok(())
}

// https://github.com/sarchar/libdragon-rs/blob/main/libdragon-sys/build.rs
