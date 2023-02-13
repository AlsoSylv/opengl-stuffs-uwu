use std::{env, fs::File, path::Path};

use gl_generator::{GlobalGenerator, Registry};

fn main() {
    let dest = env::var("OUT_DIR").unwrap();
    let mut file = File::create(Path::new(&dest).join("gl_bindings.rs")).unwrap();

    Registry::new(
        gl_generator::Api::Gl,
        (4, 5),
        gl_generator::Profile::Core,
        gl_generator::Fallbacks::All,
        [],
    )
    .write_bindings(GlobalGenerator, &mut file)
    .unwrap();
}
