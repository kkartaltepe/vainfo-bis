extern crate bindgen;

use std::env;
use std::path::PathBuf;

fn main() {
    let target = env::var("TARGET").unwrap();
    println!("cargo:rustc-link-lib=va");
    println!("cargo:rustc-link-lib=va-drm");
    println!("cargo:rustc-link-search=/usr/lib/{}", target);

    let gen_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    let bindings_file = gen_path.join("bindings.rs");

    let stat = std::fs::metadata(&bindings_file);
    match stat.err() {
        Some(err) => match err.kind() {
            std::io::ErrorKind::NotFound => {
                let bindings = bindgen::Builder::default()
                    .header("/usr/include/va/va.h")
                    .header("/usr/include/va/va_drm.h")
                    .header("/usr/include/va/va_drmcommon.h")
                    .prepend_enum_name(false)
                    .generate()
                    .expect("Unable to generate bindings");

                bindings
                    .write_to_file(&bindings_file)
                    .expect("Couldnt write bindings!");
            }
            _ => {}
        },
        None => {}
    }
}
