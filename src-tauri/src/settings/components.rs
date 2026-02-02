use crate::define_component;
use crate::i18n::locales::Locales;
use schemars::Schema;

define_component!(Password, String, "password");
define_component!(TextArea, String, "text_area");
define_component!(Switch, bool, "switch");
define_component!(Language, Option<Locales>, "language");
