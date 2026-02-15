#![warn(
    clippy::all,
    // clippy::restriction,
    clippy::pedantic,
    clippy::nursery,
    // clippy::cargo
    unused_crate_dependencies,
)]
use faust_build::code_option::CodeOption;
use faust_ui_build::file_with_ui;
use std::env;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

fn main() {
    let mut b = file_with_ui("src/faust.dsp", "src/dsp.rs");
    b.set_code_option(CodeOption::NoFaustDsp);
    b.set_code_option(CodeOption::NoLibM);
    let faust_path = env::var("FAUST_PATH").expect("FAUST_PATH mut be set");
    b.set_faust_path(format!("{faust_path}/build/bin/faust"));
    b.set_import_dir(format!("{faust_path}/libraries/"));
    b.add_code_gen_fun(|_dsp_name| {
        quote::quote! {
            // use core::prelude::rust_2024::derive;
            use core::option::Option;
            use core::iter::Iterator;
            use core::iter::ExactSizeIterator;
            use core::clone::Clone;
            #[allow(unused_imports,reason = "false positive")]
            use num_traits::Float;
        }
    });
    b.build();

    // Put `memory.x` in our output directory and ensure it's
    // on the linker search path.
    let out = &PathBuf::from(env::var_os("OUT_DIR").unwrap());
    File::create(out.join("memory.x"))
        .unwrap()
        .write_all(include_bytes!("memory.x"))
        .unwrap();
    println!("cargo:rustc-link-search={}", out.display());

    // By default, Cargo will re-run a build script whenever
    // any file in the project changes. By specifying `memory.x`
    // here, we ensure the build script is only re-run when
    // `memory.x` is changed.
    println!("cargo:rerun-if-changed=memory.x");
    
    // Re-run if FAUST_PATH environment variable changes
    println!("cargo:rerun-if-env-changed=FAUST_PATH");
    
    // Re-run if the Faust DSP source file changes
    println!("cargo:rerun-if-changed=src/faust.dsp");
}
