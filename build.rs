
use std::env;
use std::path::{PathBuf, Path};

fn main() {
  let out_dir = PathBuf::from(env::var("OUT_DIR").expect("Couldn't read `OUT_DIR` value."));
  let bindings_filename = out_dir.join("bindings.rs");
  if cfg!(feature = "force_bindgen") || file_missing(&bindings_filename) {
    let bindings = bindgen::builder()
      .header_contents("wrapper.h",r##"#include "include/SDL.h""##)
      .use_core()
      .ctypes_prefix("libc")
      .default_enum_style(bindgen::EnumVariation::Consts)
      // TODO: various whitelist and blacklist stuff goes here
      // TODO: filter what types get what impls
      .time_phases(true) // Note(Lokathor): just for fun!
      .rustfmt_bindings(true)
      .rustfmt_configuration_file(Some(PathBuf::from("rustfmt.toml")))
      .generate()
      .expect("Couldn't generate the bindings.");
    bindings.write_to_file(&bindings_filename).expect("Couldn't write the bindings file.");
  }
}

/// Say if a file is missing from the disk
fn file_missing(name: &Path) -> bool {
  std::fs::File::open(name).is_err()
}
