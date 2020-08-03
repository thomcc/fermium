#![allow(unused_imports)]
#![allow(non_snake_case)]

use std::{
  env,
  path::{Path, PathBuf},
  process::Command,
};

fn main() {
  let use_bindgen_bin = cfg!(feature = "use_bindgen_bin");
  println!("use_bindgen_bin: {}", use_bindgen_bin);

  let dynamic_link = cfg!(feature = "dynamic_link");
  let static_link = cfg!(feature = "static_link");
  println!("dynamic_link: {}", dynamic_link);
  println!("static_link: {}", static_link);
  let link_count = dynamic_link as usize + static_link as usize;
  if link_count != 1 {
    panic!(
      "You must select exactly one linking type, you selected {}",
      link_count
    );
  }

  let target = env::var("TARGET").expect("Couldn't read `TARGET`");
  println!("cargo:rustc-env=TARGET={}", target);

  if cfg!(feature = "use_bindgen_bin") {
    run_bindgen_bin();
  }

  #[cfg(all(feature = "cmake", windows))]
  {
    if cfg!(feature = "static_link") && target.contains("windows") {
      println!(
        "cargo:rustc-link-search=native={}",
        build_the_c_library().join("lib").display()
      );
    }
  }

  declare_linking();
}

fn run_bindgen_bin() {
  println!("bindgen --version: {}", {
    let mut bindgen_version = Command::new("bindgen");
    bindgen_version.arg("--version");
    let version_out =
      bindgen_version.output().expect("Couldn't execute `bindgen --version`!");
    if version_out.status.success() {
      String::from_utf8_lossy(&version_out.stdout).into_owned()
    } else {
      panic!("`bindgen --version` did not give an exit-success!");
    }
  });

  //
  let current_dir =
    env::current_dir().expect("Couldn't read the current directory.");
  let wrapper_filename = current_dir.join("wrapper.h");

  //
  let target = env::var("TARGET").expect("Couldn't read `TARGET`");
  let out_dir =
    PathBuf::from(env::var("OUT_DIR").expect("Couldn't read `OUT_DIR`"));
  let out_filename =
    format!("{}", out_dir.join(format!("SDL2-2.0.12-{}.rs", target)).display());

  // build up the whole bindgen command
  let mut bindgen = Command::new("bindgen");

  // args
  bindgen.arg("--disable-name-namespacing");
  bindgen.arg("--impl-debug");
  bindgen.arg("--impl-partialeq");
  bindgen.arg("--no-doc-comments");
  bindgen.arg("--no-prepend-enum-name");
  bindgen.arg("--no-layout-tests");
  bindgen.arg("--use-core");
  bindgen.arg("--with-derive-default");
  bindgen.arg("--with-derive-partialeq");
  bindgen.arg("--size_t-is-usize");

  // options
  bindgen.arg("--ctypes-prefix").arg("chlorine");
  bindgen.arg("--default-enum-style").arg("consts");
  bindgen.arg("--output").arg(&out_filename);
  bindgen.arg("--rust-target").arg("1.30");
  /*
  bindgen.arg("--rustfmt-configuration-file").arg(
    std::env::current_dir()
      .expect("couldn't get current directory!")
      .join("rustfmt.toml")
      .to_str()
      .expect("rustfmt.toml file path isn't valid utf8, stop that"),
  );
  */
  bindgen.arg("--whitelist-function").arg("SDL_.*");
  bindgen.arg("--whitelist-type").arg("SDL_.*");
  bindgen.arg("--whitelist-var").arg("SDL_.*");
  bindgen.arg("--whitelist-var").arg("AUDIO_.*");
  bindgen.arg("--whitelist-var").arg("SDLK_.*");

  // header
  bindgen.arg(&wrapper_filename);

  // mario kart double dash
  bindgen.arg("--");

  // clang args
  bindgen.arg("--no-warnings");
  bindgen.arg("-target").arg(&target);

  println!("executing command: {:?}", bindgen);
  let bindgen_output = bindgen.output().expect(
    "Couldn't run 'bindgen', perhaps you need to 'cargo install bindgen'?",
  );
  if bindgen_output.status.success() {
    println!("command success!")
  } else {
    println!("command failure!")
  }
  for line in String::from_utf8_lossy(&bindgen_output.stdout).lines() {
    println!("OUT:{}", line);
  }
  for line in String::from_utf8_lossy(&bindgen_output.stderr).lines() {
    println!("ERR:{}", line);
  }
  if bindgen_output.status.success() {
    // bindgen doesn't actually rustfmt it seems. we'll give it a try.
    let mut rustfmt = Command::new("rustfmt");
    rustfmt.arg(&out_filename);
    let _ = rustfmt.output();
  } else {
    panic!("The 'bindgen' command failed! (see output log for details)");
  }
}

#[cfg(all(feature = "cmake", windows))]
fn build_the_c_library() -> PathBuf {
  let target = env::var("TARGET").expect("Couldn't read `TARGET`");
  let manifest_dir = PathBuf::from(
    env::var("CARGO_MANIFEST_DIR")
      .expect("Could not read `CARGO_MANIFEST_DIR`!"),
  );
  let mut cm = cmake::Config::new(manifest_dir.join("SDL2-2.0.12"));
  cm.profile("release");
  cm.static_crt(true);
  cm.target(&target);

  if cfg!(feature = "dynamic_link") {
    cm.define("SDL_SHARED", "ON");
    cm.define("SDL_STATIC", "OFF");
  } else if cfg!(feature = "static_link") {
    cm.define("SDL_SHARED", "OFF");
    cm.define("SDL_STATIC", "ON");
  } else {
    panic!("You should have selected a link mode!");
  }

  cm.build()
}

fn declare_linking() {
  if cfg!(windows) {
    declare_win32_linking()
  } else {
    declare_sd2_config_linking()
  }
}

fn declare_win32_linking() {
  // What to link
  if cfg!(feature = "dynamic_link") {
    println!("cargo:rustc-link-lib=SDL2");
  } else {
    println!("cargo:rustc-link-lib=static=SDL2");
    // Note(Lokathor): this comes from the CMakeLists.txt, search for "Libraries
    // for Win32 native and MinGW"
    println!("cargo:rustc-link-lib=user32");
    println!("cargo:rustc-link-lib=gdi32");
    println!("cargo:rustc-link-lib=winmm");
    println!("cargo:rustc-link-lib=imm32");
    println!("cargo:rustc-link-lib=ole32");
    println!("cargo:rustc-link-lib=oleaut32");
    println!("cargo:rustc-link-lib=version");
    println!("cargo:rustc-link-lib=uuid");
    println!("cargo:rustc-link-lib=advapi32");
    println!("cargo:rustc-link-lib=setupapi");
    println!("cargo:rustc-link-lib=shell32");
  }

  // where to look
  let manifest_dir = PathBuf::from(
    env::var("CARGO_MANIFEST_DIR")
      .expect("Could not read `CARGO_MANIFEST_DIR`!"),
  );

  if cfg!(feature = "dynamic_link") {
    let sub_directory: &str = if cfg!(target_env = "gnu") {
      panic!("No provided library files for the gnu toolchain. File a PR.")
    } else if cfg!(target_env = "msvc") {
      if cfg!(target_arch = "x86") {
        r#"win32-dynamic-link-files\x86"#
      } else if cfg!(target_arch = "x86_64") {
        r#"win32-dynamic-link-files\x64"#
      } else {
        panic!("No provided library files for this CPU type.")
      }
    } else {
      panic!("Unknown 'target_env' value");
    };
    println!(
      "cargo:rustc-link-search=native={}",
      manifest_dir.join(sub_directory).display()
    );
  } else if cfg!(feature = "static_link") {
    println!("link search should have been emitted during the cmake build.");
  } else {
    panic!("You didn't select a link mode!");
  }
}

fn declare_sd2_config_linking() {
  // Verify that sdl2-config exists and supports the linking we want. The output
  // of this should go to stderr.
  let sdl2_config_usage = Command::new("sdl2-config")
    .output()
    .expect("couldn't run `sdl2-config`, please properly install SDL2.");
  assert!(!sdl2_config_usage.status.success());
  let usage_out_string = String::from_utf8_lossy(&sdl2_config_usage.stderr);
  println!("sdl2-config: {}", usage_out_string);
  let usage_words: Vec<String> =
    usage_out_string.split_whitespace().map(|s| s.to_string()).collect();
  assert!(&usage_words[0] == "Usage:", "Unexpected usage message, aborting!");
  if cfg!(feature = "dynamic_link") {
    assert!(
      usage_words.contains(&"[--libs]".to_string()),
      "This SDL2 install is not built for dynamic linking!"
    );
  }
  if cfg!(feature = "static_link") {
    assert!(
      usage_words.contains(&"[--static-libs]".to_string()),
      "This SDL2 install is not built for dynamic linking!"
    );
  }

  // Verify that the version installed is at least as much as the user is using
  // bindings for.
  let sdl2_config_version = Command::new("sdl2-config")
    .arg("--version")
    .output()
    .expect("couldn't run `sdl2-config`, please properly install SDL2.");
  assert!(sdl2_config_version.status.success());
  let version_out_string = String::from_utf8_lossy(&sdl2_config_version.stdout);
  println!("sdl2-config --version: {}", version_out_string);

  // Call sdl2-config for real and do what it says to do.
  let link_style_arg: &str = if cfg!(feature = "dynamic_link") {
    "--libs"
  } else if cfg!(feature = "static_link") {
    "--static-libs"
  } else {
    panic!("No link mode selected!");
  };
  let sd2_config_linking =
    Command::new("sdl2-config").arg(link_style_arg).output().unwrap_or_else(
      |_| panic!("Couldn't run `sdl2-config {}`.", link_style_arg),
    );
  assert!(sd2_config_linking.status.success());
  let sd2_config_linking_string: String =
    String::from_utf8_lossy(&sd2_config_linking.stdout).into_owned();
  println!("sd2_config_linking: {}", sd2_config_linking_string);
  assert!(sd2_config_linking_string.len() > 0);
  for term in sd2_config_linking_string.split_whitespace() {
    if term.starts_with("-L") {
      println!("cargo:rustc-link-search=native={}", &term[2..]);
    } else if term.starts_with("-lSDL2") {
      if cfg!(feature = "dynamic_link") {
        println!("cargo:rustc-link-lib=SDL2")
      } else if cfg!(feature = "static_link") {
        println!("cargo:rustc-link-lib=static=SDL2")
      } else {
        panic!("No link mode selected!");
      };
    } else if term.starts_with("-l") {
      // normal link
      println!("cargo:rustc-link-lib={}", &term[2..]);
    } else if term.starts_with("-Wl,-framework,") {
      // macOS framework link
      println!("cargo:rustc-link-lib=framework={}", &term[15..]);
    } else if term.starts_with("-Wl,-weak_framework,") {
      // rust doesn't seem to have "weak" framework linking so we just declare
      // a normal framework link.
      println!("cargo:rustc-link-lib=framework={}", &term[20..]);
    } else if term.starts_with("-Wl,-rpath,") {
      // I don't know why this works, but it does seem to?
      println!("cargo:rustc-env=LD_LIBRARY_PATH={}", &term[11..]);
    } else if term.starts_with("-Wl,--enable-new-dtags") {
      // Do we do anything here?
    } else if term.starts_with("-Wl,--no-undefined") {
      // Do we do anything here?
    } else if term.starts_with("-pthread") {
      // Nothing special on the Rust side
    } else {
      panic!("Unknown term: >>{}<<", term);
    }
  }
  // Note(Lokathor): If you get `sdl2-config` from the package manager instead
  // of building from source it usually won't actually give an -L term for where
  // to look for SDL2 itself. However, we can just wildly guess about where SDL2
  // probably is based on what Debian / Ubuntu do. Sane, right?
  println!("cargo:rustc-link-search=native=/usr/lib");
  println!("cargo:rustc-link-search=native=/usr/local/lib");
  if cfg!(target_arch = "x86_64")
    && cfg!(target_os = "linux")
    && cfg!(target_env = "gnu")
  {
    println!("cargo:rustc-link-search=native=/usr/lib/x86_64-linux-gnu");
  }
  if cfg!(target_arch = "aarch64")
    && cfg!(target_os = "linux")
    && cfg!(target_env = "gnu")
  {
    println!("cargo:rustc-link-search=native=/usr/lib/aarch64-linux-gnu");
  }
  if cfg!(target_arch = "arm")
    && cfg!(target_os = "linux")
    && cfg!(target_env = "gnu")
  {
    println!("cargo:rustc-link-search=native=/usr/lib/arm-linux-gnueabihf");
  }
  if cfg!(target_arch = "x86")
    && cfg!(target_os = "linux")
    && cfg!(target_env = "gnu")
  {
    println!("cargo:rustc-link-search=native=/usr/lib/i386-linux-gnu");
  }
  if cfg!(target_arch = "powerpc64")
    && cfg!(target_os = "linux")
    && cfg!(target_env = "gnu")
  {
    println!("cargo:rustc-link-search=native=/usr/lib/powerpc64le-linux-gnu");
  }
}
