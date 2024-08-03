use fluent_static_codegen::{generate, MessageBundleCodeGenerator};
use std::path::Path;
use std::{env, fs};

fn main() {
    // TODO: Workaround for https://github.com/zaytsev/fluent-static/issues/4
    println!("cargo:rerun-if-changed=res/translations");

    fs::write(
        Path::new(&env::var("OUT_DIR").unwrap()).join("l10n.rs"),
        generate("res/translations", MessageBundleCodeGenerator::new("en")).unwrap(),
    )
    .unwrap();

    #[cfg(windows)]
    {
        let mut res = winres::WindowsResource::new();
        res.set_icon("res\\windows\\space-acres.ico");
        res.compile().unwrap();
    }
}
