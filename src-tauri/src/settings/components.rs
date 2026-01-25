use schemars::Schema;
use crate::define_component;

define_component!(Password, String, "password");
define_component!(TextArea, String, "text_area");
define_component!(Switch, bool, "switch");
