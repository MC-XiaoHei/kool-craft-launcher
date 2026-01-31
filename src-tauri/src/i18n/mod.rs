use fluent_templates::static_loader;
use tap::Pipe;

static_loader! {
    pub static LOCALES = {
        locales: "../locales",
        fallback_language: "en-US",
    };
}

include!(concat!(env!("OUT_DIR"), "/generated_i18n_keys.rs"));
