use schemars::Schema;
use specta::specta;

#[macro_export]
macro_rules! define_component {
    ($name:ident, $base:ty, $component:literal) => {
        #[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize, PartialEq, Eq, specta::Type)]
        #[serde(transparent)]
        pub struct $name(#[specta(type = $base)] $base);

        impl schemars::JsonSchema for $name {
            fn schema_name() -> std::borrow::Cow<'static, str> {
                std::borrow::Cow::Borrowed(stringify!($name))
            }

            fn json_schema(generator: &mut schemars::SchemaGenerator) -> schemars::Schema {
                let base_schema = <$base>::json_schema(generator);
                $crate::settings::macros::inject(base_schema, $component)
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
    };
}

pub fn inject(mut schema: Schema, component: &str) -> Schema {
    schema.insert("component".to_string(), component.into());
    schema
}

#[macro_export]
macro_rules! evolution_chain {
    ($val:expr, $ver:expr, { $($steps:tt)* }) => {
        {
            evolution_chain!(@validate start $($steps)*);
            evolution_chain!(@run $val, $ver, $($steps)*);
        }
    };

    (@validate start $s:literal => $e:literal $b:block $(, $($tail:tt)*)?) => {
        const _: () = {
            if $e != $s + 1 {
                panic!(concat!(
                    "Invalid step ", stringify!($s), " => ", stringify!($e),
                    ". Steps must be incremental (N => N+1)."
                ));
            }
        };
        evolution_chain!(@validate next $e, $($($tail)*)?);
    };

    (@validate next $expected:literal, $s:literal => $e:literal $b:block $(, $($tail:tt)*)?) => {
        const _: () = {
            if $s != $expected {
                let expected_end = $expected + 1;
                panic!(concat!(
                    "Expected next step to start with v", stringify!($expected),
                    ", but found v", stringify!($s), " => v", stringify!($e),
                    ". Please ensure migrations are sequential (e.g., 1 => 2, 2 => 3)."
                ));
            }
            if $e != $s + 1 {
                panic!(concat!(
                    "Invalid step ", stringify!($s), " => ", stringify!($e),
                    ". Steps must be incremental (N => N+1)."
                ));
            }
        };
        evolution_chain!(@validate next $e, $($($tail)*)?);
    };

    (@validate next $expected:literal, $(,)?) => {};
    (@validate start $(,)?) => {};

    (@run $val:expr, $ver:expr, $($s:literal => $e:literal $b:block),* $(,)?) => {
        $(
            if $ver == $s {
                info!("Auto migrating v{} => v{}", $s, $e);
                { $b }
                $ver = $e;
            }
        )*
    };
}
