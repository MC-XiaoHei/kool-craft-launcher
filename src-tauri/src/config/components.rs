use schemars::Schema;

pub fn inject(mut schema: Schema, component: &str) -> Schema {
    schema.insert("component".to_string(), component.into());
    schema
}

#[macro_export]
macro_rules! define_component {
    ($name:ident, $base:ty, $component:literal) => {
        #[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
        #[serde(transparent)]
        pub struct $name($base);

        impl schemars::JsonSchema for $name {
            fn schema_name() -> std::borrow::Cow<'static, str> {
                std::borrow::Cow::Borrowed(stringify!($name))
            }

            fn json_schema(generator: &mut schemars::SchemaGenerator) -> schemars::Schema {
                let base_schema = <$base>::json_schema(generator);
                $crate::config::components::inject(base_schema, $component)
            }
        }

        impl std::ops::Deref for $name {
            type Target = $base;
            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }

        impl std::ops::DerefMut for $name {
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.0
            }
        }

        impl From<$base> for $name {
            fn from(v: $base) -> Self {
                Self(v)
            }
        }

        impl std::fmt::Display for $name
        where
            $base: std::fmt::Display,
        {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                self.0.fmt(f)
            }
        }
    };
}

define_component!(Password, String, "password");
define_component!(TextArea, String, "text_area");
define_component!(Switch, bool, "switch");
