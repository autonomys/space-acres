use fluent_static_codegen::{generate, MessageBundleCodeGenerator};
use std::path::Path;
use std::{env, fs};

fn main() {
    let mut generated =
        generate("res/translations", MessageBundleCodeGenerator::new("en")).unwrap();
    // This is a hack for making sure `NUMBER()` is supported, see https://github.com/projectfluent/fluent-rs/pull/353#issuecomment-2266336661
    {
        if !generated.contains("; bundle }") {
            panic!("Unexpected generated contents: {generated}");
        }
        generated = generated.replace(
            "; bundle }",
            // TODO: Should have been `bundle.add_builtins().unwrap();`, but https://github.com/projectfluent/fluent-rs/issues/368
            r#"; bundle.add_function("NUMBER", super::number).unwrap(); bundle }"#,
        );
    }
    fs::write(
        Path::new(&env::var("OUT_DIR").unwrap()).join("l10n.rs"),
        generated,
    )
    .unwrap();

    #[cfg(windows)]
    {
        let mut res = winres::WindowsResource::new();
        res.set_icon("res\\windows\\space-acres.ico");
        res.compile().unwrap();
    }

    relm4_icons_build::bundle_icons(
        "icon_names.rs",
        Some("xyz.autonomys.space_acres"),
        None::<&str>,
        None::<&str>,
        [
            "ssd",
            "size-horizontally",
            "cross",
            "checkmark",
            "wallet2",
            "warning",
            "puzzle-piece",
            "pause",
            "menu-large",
            "processor",
            "speedometer2",
            "speedometer3",
            "speedometer4",
            "grid-filled",
        ],
    );
}
