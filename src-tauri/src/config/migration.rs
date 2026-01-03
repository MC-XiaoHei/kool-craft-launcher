#[macro_export]
macro_rules! migration_chain {
    ($val:expr, $ver:expr, { $($steps:tt)* }) => {
        {
            migration_chain!(@validate start $($steps)*);
            migration_chain!(@run $val, $ver, $($steps)*);
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
        migration_chain!(@validate next $e, $($($tail)*)?);
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
        migration_chain!(@validate next $e, $($($tail)*)?);
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
