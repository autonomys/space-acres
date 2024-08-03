mod messages {
    #![allow(clippy::all)]

    use fluent_static::fluent_bundle::{FluentArgs, FluentValue};
    // TODO: This is used inside the generated file and is a hack for
    //  https://github.com/projectfluent/fluent-rs/issues/368
    fn number<'a>(positional: &[FluentValue<'a>], named: &FluentArgs) -> FluentValue<'a> {
        let Some(FluentValue::Number(n)) = positional.first() else {
            return FluentValue::Error;
        };

        let mut n = n.clone();
        n.options.merge(named);
        if let Some(maximum_fraction_digits) = n.options.maximum_fraction_digits {
            let multiplier = 10f64.powf(maximum_fraction_digits as f64);
            n.value = (n.value * multiplier).round() / multiplier;
        }

        FluentValue::Number(n)
    }

    pub use messages::*;
    include!(concat!(env!("OUT_DIR"), "/l10n.rs"));
}

use fluent_langneg::{
    convert_vec_str_to_langids, convert_vec_str_to_langids_lossy, negotiate_languages,
    NegotiationStrategy,
};
use fluent_static::fluent_bundle::FluentError;
use fluent_static::{LanguageSpec, Message};
use messages::MessagesBundle;
use std::iter;
use std::sync::LazyLock;
use tracing::error;

/// Translations for local language on this machine
pub static T: LazyLock<MessagesBundle> = LazyLock::new(|| {
    let all_languages = MessagesBundle::all_languages();
    let available = convert_vec_str_to_langids(all_languages)
        .expect("Translations are all statically valid due to code generation; qed");
    let requested = convert_vec_str_to_langids_lossy(
        sys_locale::get_locales().chain(iter::once("en".to_string())),
    );

    let selected_languages =
        negotiate_languages(&requested, &available, None, NegotiationStrategy::Filtering)
            .into_iter()
            .cloned()
            .collect::<Vec<_>>();
    let primary_language = selected_languages
        .first()
        .expect("Not empty due to fallback at the end of requested list; qed");
    let primary_language = all_languages
        .iter()
        .zip(&available)
        .find_map(|(str_language, language)| {
            (language == primary_language).then(|| LanguageSpec::new(str_language.to_string()))
        })
        .expect("Not empty due to fallback at the end of requested list; qed");

    MessagesBundle::from(primary_language)
});

pub trait AsDefaultStr {
    /// Get a `&str` and use placeholder message value in case of an error
    fn as_str(&self) -> &str;

    /// Get a `String` and use placeholder message value in case of an error
    fn to_string(self) -> String;
}

impl AsDefaultStr for Result<Message<'static>, FluentError> {
    fn as_str(&self) -> &str {
        self.as_deref()
            .inspect_err(|error| {
                error!(%error, "Fluent formatting error");
            })
            .unwrap_or("<Translation error>")
    }

    fn to_string(self) -> String {
        self.as_str().to_string()
    }
}
