// ---------------- [ File: src/rand_construct_env.rs ]
crate::ix!();

pub trait RandConstructEnvironment {
    fn create_random<R>() -> R
    where
        R: RandConstructEnumWithEnv,
        Self: RandConstructProbabilityMapProvider<R> + Sized,
    {
        R::random_with_env::<Self>()
    }

    fn create_random_uniform<R>() -> R
    where
        R: RandConstructEnumWithEnv,
        Self: RandConstructProbabilityMapProvider<R> + Sized,
    {
        R::random_uniform_with_env::<Self>()
    }
}

#[macro_export]
macro_rules! rand_construct_env {
    ($provider:ident => $enum:ty { $($variant:ident => $weight:expr),* $(,)? }) => {
        impl $crate::RandConstructProbabilityMapProvider<$enum> for $provider {
            fn probability_map() -> std::sync::Arc<std::collections::HashMap<$enum, f64>> {
                use once_cell::sync::Lazy;
                static PROBABILITY_MAP: Lazy<std::sync::Arc<std::collections::HashMap<$enum, f64>>> = Lazy::new(|| {
                    let mut map = std::collections::HashMap::new();
                    $(
                        map.insert(<$enum>::$variant, $weight);
                    )*
                    std::sync::Arc::new(map)
                });
                std::sync::Arc::clone(&PROBABILITY_MAP)
            }

            fn uniform_probability_map() -> std::sync::Arc<std::collections::HashMap<$enum, f64>> {
                use once_cell::sync::Lazy;
                static UNIFORM_PROBABILITY_MAP: Lazy<std::sync::Arc<std::collections::HashMap<$enum, f64>>> = Lazy::new(|| {
                    let mut map = std::collections::HashMap::new();
                    $(
                        map.insert(<$enum>::$variant, 1.0);
                    )*
                    std::sync::Arc::new(map)
                });
                std::sync::Arc::clone(&UNIFORM_PROBABILITY_MAP)
            }
        }
    };
}
