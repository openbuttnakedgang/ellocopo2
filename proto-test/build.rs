use std::env;
use std::fs::read_to_string;
use std::fs::write;
use std::path::Path;
use std::process::Command;

fn main() {

    // Obtain scheme path from evar
    let evar = env::var("ELLOCOPO2_SCHEME_PATH");
    let path = match &evar {
        Ok(path) => Path::new(path),
        Err(_) => Path::new("../scheme.json"),
    };

    let txt = read_to_string(path).unwrap();
    let txt = ellocopo2_codegen::generate(&txt);
    let path = Path::new(&env::var("OUT_DIR").unwrap()).join("codegen.rs");
    let _ = write(path.clone(), txt);

    // rustfmt
    Command::new("rustfmt")
        .arg(path)
        .output()
        .expect("rustfmt: failed to execute");

    println!("cargo:rerun-if-changed=scheme.json");
}
