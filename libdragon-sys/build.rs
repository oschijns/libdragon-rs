use error_chain::error_chain;
use std::{
    env,
    fs::{self, File},
    path::PathBuf,
    process::{exit, Command},
};

use execute::Execute;
use filetime::FileTime;

error_chain! {
    foreign_links {
        Io(std::io::Error);
        HttpRequest(reqwest::Error);
        Zip(zip::result::ZipError);
    }
}

#[derive(Debug)]
struct Cb;

impl bindgen::callbacks::ParseCallbacks for Cb {
    fn process_comment(&self, comment: &str) -> Option<String> {
        //eprintln!("cmt: {:?}", comment);
        Some(doxygen_rs::transform(comment))
    }
}

const TOOLCHAIN_URL: &str = "https://github.com/sarchar/libdragon/releases/download/toolchain-continuous-prerelease/gcc-toolchain-mips64-linux.zip";

#[tokio::main]
async fn main() -> Result<()> {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let src_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let libdragon_dir = src_dir.clone().join("libdragon");
    let toolchain_dir = out_dir.clone().join("toolchain");

    // configure N64_INST for building libdragon and the toolchain
    env::set_var("N64_INST", toolchain_dir.display().to_string());
    println!("cargo:rustc-env=N64_INST={}", toolchain_dir.display());
    eprintln!("N64_INST={}", toolchain_dir.display());

    if !cfg!(feature = "buildtoolchain") {
        let download_dir = out_dir.clone();
        let download_file = download_dir.clone().join("gcc-toolchain-mips64-x86_64.zip");

        // Don't download toolchain file if it exists
        if !download_file.exists() {
            let url = TOOLCHAIN_URL;
            eprintln!("Downloading gcc-toolchain-mips64-linux.zip ...");
            let response = reqwest::get(url).await?;
            let mut content = std::io::Cursor::new(response.bytes().await?);

            // the zip we actually want is inside the downloaded zip, so save the download to a tempfile
            let tmp_dir = tempfile::Builder::new().prefix("libdragon-rs").tempdir()?;
            let tmp_file = tmp_dir.path().join("gcc-toolchain-mips64-linux.zip");
            {
                let mut fp: File = File::create(tmp_file.clone())?;
                eprintln!("Tempfile={:?}", fp);
                let _ = std::io::copy(&mut content, &mut fp);
            }

            eprintln!(
                "Copying gcc-toolchain-mips64-x86_64.zip to {}",
                download_dir.display()
            );
            let mut archive = zip::ZipArchive::new(File::open(tmp_file)?)?;
            let mut toolchain_file = archive.by_name("gcc-toolchain-mips64-x86_64.zip")?;

            let mut final_fp = File::create(download_file.clone())?;
            eprintln!("Output={:?}", final_fp);
            let _ = std::io::copy(&mut toolchain_file, &mut final_fp);
        } else {
            eprintln!("Skipping download");
        }

        // Don't extract the archive if the toolchain directory exists
        if !toolchain_dir.exists() {
            let archive_file = File::open(download_file.clone())?;
            let mut archive = zip::ZipArchive::new(archive_file)?;
            archive.extract(toolchain_dir.clone())?;
            eprintln!("Toolchain extracted to {}", toolchain_dir.display());
        } else {
            eprintln!("Skipping extract");
        }
    } else {
        // build toolchain
        // tell cargo that if build-toolchain.sh script changes we should re-run this script
        println!("cargo:rerun-if-changed=libdragon/tools/build-toolchain.sh");
        println!("cargo:rerun-if-changed=libdragon");

        // create the build directory under out/
        let build_toolchain_dir = out_dir.clone().join("build-toolchain");
        fs::create_dir_all(&build_toolchain_dir).expect("Error creating build directory");

        // if {out}/mips64-libdragon-elf/bin/... doesn't exist OR if
        // <xyz> is newer than <xyz>, build toolchain
        // build the toolchain. execute from out for the build
        let build_toolchain_script = libdragon_dir
            .clone()
            .join("tools")
            .join("build-toolchain.sh");
        let gcc = toolchain_dir
            .clone()
            .join("bin")
            .join("mips64-libdragon-elf-gcc");
        let need_toolchain_build = !fs::metadata(gcc).is_ok_and(|metadata| {
            let gcc_time = FileTime::from_last_modification_time(&metadata);
            let build_script_time = FileTime::from_last_modification_time(
                &fs::metadata(build_toolchain_script.clone()).unwrap(),
            );
            build_script_time <= gcc_time
        });
        if need_toolchain_build {
            let mut build_toolchain = Command::new("bash");
            build_toolchain.arg(build_toolchain_script.into_os_string());
            build_toolchain.current_dir(build_toolchain_dir.into_os_string());
            let output = build_toolchain.execute_output().unwrap();
            if let Some(exit_code) = output.status.code() {
                if exit_code != 0 {
                    eprintln!("There was an error building the mips64-libdragon-elf toolchain");
                    exit(1);
                }
            } else {
                eprintln!("Build incomplete");
                exit(1);
            }
        } else {
            eprintln!("Skipping toolchain build");
        }
    }

    // Create the build directory
    let libdragon_build_dir = out_dir.clone().join("libdragon_build");
    let mut mkdir = Command::new("mkdir");
    mkdir.arg("-p").arg(&libdragon_build_dir);
    let _ = mkdir.execute_check_exit_status_code(0); // Let's hope this doesn't ever fail

    // build libdragon
    let mut make = Command::new("make");
    make.arg("-C")
        .arg(libdragon_dir.clone().into_os_string())
        .current_dir(&libdragon_build_dir)
        .arg("libdragon")
        .arg("tools")
        .arg("-j")
        .arg("4");
    eprintln!("make: {:?}", make);
    let make_output = make.execute_output().unwrap();
    if let Some(exit_code) = make_output.status.code() {
        if exit_code != 0 {
            eprintln!("There was an error building libdragon");
            eprintln!("stdout: {}", String::from_utf8(make_output.stdout).unwrap());
            eprintln!("stderr: {}", String::from_utf8(make_output.stderr).unwrap());
            exit(1);
        }
        eprintln!("make output: {:?}", make_output);
    }

    // install libdragon and tools
    let mut install = Command::new("make");
    install
        .arg("-C")
        .arg(libdragon_dir.clone().into_os_string())
        .current_dir(&libdragon_build_dir)
        .arg("install")
        .arg("tools-install");
    if install.execute_check_exit_status_code(0).is_err() {
        eprintln!("There was an error installing libdragon");
        exit(1);
    }

    // link against libdragon.a and libdragonsys.a
    println!(
        "cargo:rustc-link-search=native={}/mips64-libdragon-elf/lib",
        toolchain_dir.display()
    );
    println!("cargo:rustc-link-lib=static=dragon");
    println!("cargo:rustc-link-lib=static=dragonsys");

    println!(
        "cargo:rustc-link-search=native={}/lib/gcc/mips64-libdragon-elf/14.1.0",
        toolchain_dir.display()
    );
    println!("cargo:rustc-link-lib=static=c");
    println!("cargo:rustc-link-lib=static=g");
    println!("cargo:rustc-link-lib=static=nosys");
    println!("cargo:rustc-link-lib=static=gcc");
    println!("cargo:rustc-link-lib=static=m");

    let static_fns_path = out_dir.clone().join("static_fns.c");

    let bindings = bindgen::Builder::default()
        .clang_arg(format!(
            "-I{}/mips64-libdragon-elf/include",
            toolchain_dir.display()
        ))
        .clang_args(&["-target", "mips-nintendo64-none", "-mabi=n32", "-DN64"])
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
        .arg(toolchain_dir.clone().join("include"))
        .arg("-I")
        .arg(
            toolchain_dir
                .clone()
                .join("mips64-libdragon-elf")
                .join("include"),
        );
    // With gcc...
    //let static_fns_obj_path = out_dir.clone().join("static_fns.o");
    //let mut compile_fns = Command::new(toolchain_dir.clone().join("bin").join("mips64-libdragon-elf-gcc"));
    //compile_fns.arg("-march=vr4300").arg("-mtune=vr4300")
    //           .arg("-mabi=n32").arg("-mno-abicalls").arg("-G").arg("0").arg("-mno-gpopt")
    //           .arg("-falign-functions=32")
    //           .arg("-ffunction-sections").arg("-fdata-sections").arg("-g")
    //           .arg(format!("-ffile-prefix-map={}=", out_dir.clone().display()))
    //           .arg("-DN64").arg("-Wall").arg("-std=gnu99")
    //           .arg("-O2")
    //           .arg("-c")
    //           .arg("-o")
    //           .arg(&static_fns_obj_path)
    //           .arg(static_fns_path)
    //           .arg("-I")
    //           .arg(src_dir.clone())
    //           .arg("-I")
    //           .arg(toolchain_dir.clone().join("include"))
    //           .arg("-I")
    //           .arg(toolchain_dir.clone().join("mips64-libdragon-elf").join("include"));

    eprintln!("compile: {:?}", compile_fns);
    if compile_fns.execute_check_exit_status_code(0).is_err() {
        eprintln!("Could not compile static_fns.c");
        exit(1);
    }

    // Add the static_fns.o object to an archive
    let mut add_archive = Command::new(
        toolchain_dir
            .clone()
            .join("bin")
            .join("mips64-libdragon-elf-ar"),
    );
    add_archive
        .arg("-crus")
        .arg(
            toolchain_dir
                .clone()
                .join("mips64-libdragon-elf")
                .join("lib")
                .join("libextern.a"),
        )
        .arg(static_fns_obj_path);
    if add_archive.execute_check_exit_status_code(0).is_err() {
        eprintln!("Could not add static_fns.o to libdragon.a");
        exit(1);
    }

    // And link to the archive
    println!("cargo:rustc-link-lib=static=extern");

    // set vars for parent crates
    println!("cargo:n64_inst={}", toolchain_dir.display());
    println!(
        "cargo:n64_includedir={}/mips64-libdragon-elf/inlude",
        toolchain_dir.display()
    );
    println!(
        "cargo:n64_libdir={}/mips64-libdragon-elf/lib",
        toolchain_dir.display()
    );
    println!("cargo:linker_script={}/n64.ld", libdragon_dir.display());
    println!("cargo:rsp_linker_script={}/rsp.ld", libdragon_dir.display());
    println!(
        "cargo:toolchain_bin={}/bin/mips64-libdragon-elf-",
        toolchain_dir.display()
    );
    println!("cargo:n64_tooldir={}/bin", toolchain_dir.display());
    println!(
        "cargo:header={}/mips64-libdragon-elf/lib/header",
        toolchain_dir.display()
    );

    Ok(())
}
