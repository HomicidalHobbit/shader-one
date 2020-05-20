#![allow(dead_code)]

use ansi_term::Colour::Yellow;
use bindgen::builder;
use fs_extra::dir::{copy, CopyOptions};
use rm_rf::ensure_removed;
use std::env;
use std::path::Path;
use std::process::Command;
use std::{fs, io};

#[cfg(target_os = "linux")]
const PLATFORM: &str = "linux";
#[cfg(target_os = "macos")]
const PLATFORM: &str = "macos";
#[cfg(target_os = "windows")]
const PLATFORM: &str = "win64";

struct Config<'a> {
    root: &'a str,
    build_dir: &'a Path,
    lib_dir: &'a Path,
    include_dir: &'a Path,
    built_libs: &'a Path,
    built_libs64: &'a Path,
    llvm_path: &'a Path,
}

fn main() {
    let root = env::var("CARGO_MANIFEST_DIR").unwrap();
    enable_ansi();

    let include_dir = Path::new(&root).join("include");
    let build_dir = Path::new(&root).join("build");
    let lib_dir = Path::new(&root).join("lib").join(PLATFORM);
    let built_libs = build_dir.join("lib");
    let built_libs64 = build_dir.join("lib64");
    let llvm_path = Path::new(&root).join("llvm");

    let config = Config {
        root: &root,
        build_dir: &build_dir,
        lib_dir: &lib_dir,
        include_dir: &include_dir,
        built_libs: &built_libs,
        built_libs64: &built_libs64,
        llvm_path: &llvm_path,
    };

    create_if_not_exist(&lib_dir);
    create_if_not_exist(&include_dir);

    let mut lc = llvm_path.join("llvm-config");
    if cfg!(target_os = "windows") {
        lc = llvm_path.join("llvm-config.exe");
        env::set_var("LIBCLANG_PATH", llvm_path.to_str().unwrap());
    }
    env::set_var("LLVM_CONFIG_PATH", lc.to_str().unwrap());

    build_llvm(&config);
    build_glslang(&config);
    build_spirv_cross(&config);
    build_glfw(&config);
    build_freetype(&config);
    build_vulkan(&config);

    let toolchain = find_toolchain();

    cc::Build::new()
        //       .compiler("C:\\Program Files (x86)\\Microsoft Visual Studio\\2017\\BuildTools\\VC\\Tools\\MSVC\\14.16.27023\\bin\\Hostx64\\x64\\cl.exe")
        .compiler(toolchain.0)
        .cpp(true)
        .define("ENABLE_OPT", "1")
        .include(&include_dir)
        .include(toolchain.1)
        //        .include("C:\\Program Files (x86)\\Microsoft Visual Studio\\2017\\BuildTools\\VC\\Tools\\MSVC\\14.16.27023\\include")
        .include("C:\\Program Files (x86)\\Windows Kits\\10\\Include\\10.0.18362.0\\ucrt")
        .flag_if_supported("-std=c++11")
        .flag_if_supported("/EHsc")
        .flag_if_supported("-fPIC")
        .file("src/spirv.cpp")
        .file("src/book.cpp")
        .file("src/keywords.cpp")
        .compile("spirvwrapper");

    if cfg!(target_os = "macos") {
        println!("cargo:rustc-link-lib=framework=Cocoa");
        println!("cargo:rustc-link-lib=framework=IOKit");
        println!("cargo:rustc-link-lib=dylib=c++");
    }

    if cfg!(target_os = "windows") {
        println!("cargo:rustc-link-lib=dylib=gdi32");
        println!("cargo:rustc-link-lib=dylib=shell32");
        println!("cargo:rustc-link-lib=dylib=user32");
    }

    if cfg!(target_os = "linux") {
        println!("cargo:rustc-link-lib=static=stdc++");
    }

    println!("cargo:rustc-link-search=native={}", lib_dir.display());

    println!("cargo:rustc-link-lib=static=glslang");
    println!("cargo:rustc-link-lib=static=HLSL");
    println!("cargo:rustc-link-lib=static=OGLCompiler");
    println!("cargo:rustc-link-lib=static=OSDependent");
    println!("cargo:rustc-link-lib=static=SPIRV");
    println!("cargo:rustc-link-lib=static=spirvwrapper");
    println!("cargo:rustc-link-lib=static=SPIRV-Tools");
    println!("cargo:rustc-link-lib=static=SPIRV-Tools-opt");
    println!("cargo:rustc-link-lib=static=spirv-cross-core");
    println!("cargo:rustc-link-lib=static=spirv-cross-glsl");
    println!("cargo:rustc-link-lib=static=spirv-cross-hlsl");
    println!("cargo:rustc-link-lib=static=spirv-cross-msl");
    println!("cargo:rustc-link-lib=static=glfw3");
    println!("cargo:rustc-link-lib=static=freetype");
    //println!("cargo:rustc-link-lib=static=SPVRemapper");
}

fn clone_repository(name: &str, path: &Path) {
    let message = format!("Cloning '{}' to {}", name, path.display());
    eprintln!("{}", Yellow.paint(message));
    Command::new("git")
        .arg("clone")
        .arg(name)
        .arg(path)
        .output()
        .expect("Can't spawn Git!!");
}

fn create_if_not_exist(path: &Path) {
    match fs::create_dir_all(path) {
        Ok(_) => {}
        Err(_) => {}
    }
}

fn copy_multi_to_lib(names: Vec<&str>, config: &Config) -> bool {
    for name in &names {
        if !copy_to_lib(name, config) {
            return false;
        }
    }
    true
}

fn copy_to_lib(name: &str, config: &Config) -> bool {
    let mut result = true;
    let final_name = get_platform_lib_name(name);
    match fs::copy(
        config.built_libs.join(&final_name),
        config.lib_dir.join(&final_name),
    ) {
        Ok(_) => {}
        Err(_) => {
            eprintln!("final_name: {}", &final_name);
            eprintln!("src: {}", config.built_libs.join(&final_name).display());
            eprintln!("dest: {}", config.lib_dir.join(final_name).display());
            result = false;
        }
    }
    result
}

#[cfg(target_os = "macos")]
fn get_platform_lib_name(name: &str) -> String {
    let mut final_name = String::from("lib");
    final_name.push_str(name);
    final_name.push_str(".a");
    final_name
}

#[cfg(target_os = "linux")]
fn get_platform_lib_name(name: &str) -> String {
    let mut final_name = String::from("lib");
    final_name.push_str(name);
    final_name.push_str(".a");
    final_name
}

#[cfg(target_os = "windows")]
fn get_platform_lib_name(name: &str) -> String {
    let mut final_name = String::from(name);
    final_name.push_str(".lib");
    final_name
}

#[cfg(not(target_os = "windows"))]
fn enable_ansi() {}

#[cfg(target_os = "windows")]
fn enable_ansi() {
    ansi_term::enable_ansi_support().unwrap();
}

fn build_glslang(config: &Config) {
    match fs::metadata(config.lib_dir.join(get_platform_lib_name("glslang"))) {
        Err(_) => {
            let glslang = Path::new(config.root).join("glslang");
            clone_repository("git@github.com:KhronosGroup/glslang.git", &glslang);
            let tools = glslang.join("External").join("spirv-tools");
            clone_repository("git@github.com:KhronosGroup/SPIRV-Tools.git", &tools);
            let headers = tools.join("external").join("spirv-headers");
            clone_repository("git@github.com:KhronosGroup/SPIRV-Headers.git", &headers);

            create_if_not_exist(config.build_dir);

            cmake_config("glslang", config)
                .define("SPIRV_WERROR", "OFF")
                .define("SPIRV_SKIP_EXECUTABLES", "ON")
                .define("SPIRV_SKIP_TESTS", "ON")
                .define("SPIRV_HEADERS_SKIP_EXAMPLES", "ON")
                .define("SPIRV_HEADERS_SKIP_INSTALL", "ON")
                .out_dir(config.build_dir)
                .build();

            copy_multi_to_lib(
                vec![
                    "glslang",
                    "HLSL",
                    "OGLCompiler",
                    "OSDependent",
                    "SPIRV",
                    "SPVRemapper",
                    "SPIRV-Tools",
                    "SPIRV-Tools-opt",
                    "SPIRV-Tools-link",
                    "SPIRV-Tools-reduce",
                ],
                &config,
            );

            let mut options = CopyOptions::new();
            options.overwrite = true;
            copy(
                config.build_dir.join("include").join("glslang"),
                config.include_dir,
                &options,
            )
            .unwrap();

            copy(
                config.build_dir.join("include").join("spirv-tools"),
                config.include_dir,
                &options,
            )
            .unwrap();

            ensure_removed(config.build_dir).unwrap();
            ensure_removed(headers).unwrap();
            ensure_removed(tools).unwrap();
            ensure_removed(glslang).unwrap();
        }

        Ok(_) => {}
    }
}

fn build_spirv_cross(config: &Config) {
    match fs::metadata(
        config
            .lib_dir
            .join(get_platform_lib_name("spirv-cross-core")),
    ) {
        Err(_) => {
            let spirv_cross = Path::new(config.root).join("spirv-cross");
            clone_repository("git@github.com:KhronosGroup/SPIRV-Cross.git", &spirv_cross);

            create_if_not_exist(config.build_dir);

            cmake_config("spirv-cross", config).build();

            copy_multi_to_lib(
                vec![
                    "spirv-cross-core",
                    "spirv-cross-glsl",
                    "spirv-cross-msl",
                    "spirv-cross-hlsl",
                ],
                &config,
            );

            let mut options = CopyOptions::new();
            options.overwrite = true;
            copy(
                config.build_dir.join("include").join("spirv_cross"),
                config.include_dir,
                &options,
            )
            .unwrap();
            ensure_removed(config.build_dir).unwrap();
            ensure_removed(&spirv_cross).unwrap();
        }

        Ok(_) => {}
    }
}

fn build_glfw(config: &Config) {
    match fs::metadata(config.lib_dir.join(get_platform_lib_name("glfw3"))) {
        Err(_) => {
            let glfw = Path::new(config.root).join("glfw");
            clone_repository("git@github.com:glfw/glfw.git", &glfw);

            create_if_not_exist(config.build_dir);

            let mut glfwc = cmake_config("glfw", config);
            glfwc.build();

            copy_to_lib("glfw3", &config);

            let mut options = CopyOptions::new();
            options.overwrite = true;
            copy(
                config.build_dir.join("include").join("GLFW"),
                config.include_dir,
                &options,
            )
            .unwrap();

            let root = config.include_dir.join("GLFW");

            let mut b = builder();
            b = add_header_to_bindings(b, &root.join("glfw3.h"));
            b = inject_to_bindings(b);

            let bindings = b.generate().unwrap();
            bindings
                .write_to_file(Path::new(config.root).join("src").join("glfw.rs"))
                .unwrap();

            ensure_removed(config.build_dir).unwrap();
            ensure_removed(&glfw).unwrap();
        }

        Ok(_) => {}
    }
}

fn build_freetype(config: &Config) {
    match fs::metadata(config.lib_dir.join(get_platform_lib_name("freetype"))) {
        Err(_) => {
            let freetype = Path::new(config.root).join("freetype");
            clone_repository("git://git.sv.nongnu.org/freetype/freetype2.git", &freetype);

            create_if_not_exist(config.build_dir);

            cmake_config("freetype", config)
                .define("CMAKE_BUILD_TYPE", "Release")
                .define("WITH_BZip2", "OFF")
                .define("WITH_HarfBuzz", "OFF")
                .define("WITH_PNG", "OFF")
                .define("WITH_ZLIB", "OFF")
                .build();

            copy_to_lib("freetype", &config);

            let mut options = CopyOptions::new();
            options.overwrite = true;
            copy(
                config.build_dir.join("include").join("freetype2"),
                config.include_dir,
                &options,
            )
            .unwrap();

            let mut inc = String::from("-I");
            let root = config.include_dir.join("freetype2");
            inc.push_str(root.to_str().unwrap());

            let mut b = builder()
                .clang_arg(inc)
                .raw_line("#![allow(improper_ctypes)]");

            b = add_header_to_bindings(b, &root.join("ft2build.h"));
            b = add_header_to_bindings(b, &root.join("freetype").join("freetype.h"));
            b = inject_to_bindings(b);

            let bindings = b.generate().unwrap();
            bindings
                .write_to_file(Path::new(config.root).join("src").join("freetype.rs"))
                .unwrap();

            ensure_removed(config.build_dir).unwrap();
            ensure_removed(&freetype).unwrap();
        }

        Ok(_) => {}
    }
}

fn build_llvm(config: &Config) {
    let mut llvm_check = config.llvm_path.join("llvm-config");
    if cfg!(target_os = "windows") {
        llvm_check = config.llvm_path.join("libclang.dll");
    }

    match fs::metadata(llvm_check) {
        Err(_) => {
            let llvm = Path::new(config.root).join("llvm-project");
            clone_repository("git@github.com:llvm/llvm-project.git", &llvm);
            create_if_not_exist(config.build_dir);
            create_if_not_exist(config.llvm_path);

            cmake_config("llvm-project/llvm", config)
                .define("LLVM_ENABLE_PROJECTS", "clang;libcxx;libcxxabi")
                .build();

            if cfg!(target_os = "windows") {
                fs::copy(
                    config.build_dir.join("bin").join("llvm-config.exe"),
                    config.llvm_path.join("llvm-config.exe"),
                )
                .unwrap();

                fs::copy(
                    config.build_dir.join("bin").join("libclang.dll"),
                    config.llvm_path.join("libclang.dll"),
                )
                .unwrap();
            } else {
                fs::copy(
                    config.build_dir.join("bin").join("llvm-config"),
                    config.llvm_path.join("llvm-config"),
                )
                .unwrap();
            }
            ensure_removed(config.build_dir).unwrap();
            ensure_removed(&llvm).unwrap();
        }

        Ok(_) => {}
    }
}

fn build_vulkan(config: &Config) {
    let p = Path::new(config.root);
    match fs::metadata(p.join("src").join("vulkan.rs")) {
        Err(_) => {
            let vh = p.join("vulkan-headers");
            clone_repository("git@github.com:KhronosGroup/Vulkan-Headers.git", &vh);
            create_if_not_exist(config.build_dir);

            let mut options = CopyOptions::new();
            options.overwrite = true;
            copy(
                vh.join("include").join("vulkan"),
                config.include_dir,
                &options,
            )
            .unwrap();

            let root = config.include_dir.join("vulkan");

            let mut b = builder();
            b = add_header_to_bindings(b, &root.join("vulkan_core.h"));
            b = inject_to_bindings(b);

            let bindings = b.generate().unwrap();
            bindings
                .write_to_file(Path::new(config.root).join("src").join("vulkan.rs"))
                .unwrap();
        }

        Ok(_) => {}
    }
}

fn inject_to_bindings(mut builder: bindgen::Builder) -> bindgen::Builder {
    let statements = vec![
        "#![allow(non_camel_case_types)]",
        "#![allow(non_upper_case_globals)]",
        "#![allow(dead_code)]",
        "#![allow(non_snake_case)]",
        "#![allow(unused_imports)]",
    ];

    for statement in statements {
        builder = builder.raw_line(statement);
    }
    builder
}

fn add_header_to_bindings(mut builder: bindgen::Builder, header: &Path) -> bindgen::Builder {
    builder = builder.header(String::from(header.to_str().unwrap()));
    builder
}

fn cmake_config(name: &str, config: &Config) -> cmake::Config {
    let mut ck = cmake::Config::new(name);
    if cfg!(target_os = "windows") {
        ck.profile("Release")
            .generator("Visual Studio 15 2017")
            .static_crt(true)
            .cxxflag("/MP")
            .cxxflag("/EHsc")
            .out_dir(config.build_dir);
    } else {
        ck.profile("Release").out_dir(config.build_dir);
    }
    ck
}

fn find_toolchain() -> (String, String) {
    let root = Path::new("C:\\Program Files (x86)\\Microsoft Visual Studio\\2017\\BuildTools\\VC");
    let kit = Path::new("C:\\Program Files (x86)\\Windows Kits\\10\\Include");
    let ver = root
        .join("Auxiliary")
        .join("Build")
        .join("Microsoft.VCToolsVersion.default.txt");
    let version = fs::read_to_string(ver);
    match version {
        Ok(v) => {
            let x: &[_] = &['\r', '\n'];
            let compiler = root
                .join("Tools")
                .join("MSVC")
                .join(v.trim_end_matches(x))
                .join("bin")
                .join("Hostx64")
                .join("x64")
                .join("cl.exe");

            let include = root
                .join("Tools")
                .join("MSVC")
                .join(v.trim_end_matches(x))
                .join("include");

            match fs::read_dir(kit) {
                Ok(e) => {
                    let entries = e
                        .map(|res| res.map(|a| a.path()))
                        .collect::<Result<Vec<_>, io::Error>>();
                    entries.unwrap().sort()
                }
                Err(_) => {
                    panic!("Cannot locate Windows 10 SDK!!");
                }
            }
            return (
                compiler.to_string_lossy().into_owned(),
                include.to_string_lossy().into_owned(),
            );
        }
        Err(_) => {
            panic!("Cannot locate VS2017 Build Tools!!");
        }
    }
}
