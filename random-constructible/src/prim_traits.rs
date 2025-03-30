// ---------------- [ File: src/prim_traits.rs ]
crate::ix!();

// Macro to implement RandConstruct for floating-point types
macro_rules! impl_rand_construct_for_float {
    ($($t:ty),*) => {
        $(
            impl RandConstruct for $t {
                fn random() -> Self {
                    rand::random::<$t>()
                }
                fn uniform() -> Self {
                    let mut rng = rand::thread_rng();
                    rng.gen_range(0.0 as $t .. 1.0 as $t)
                }
                fn random_with_rng<R: Rng + ?Sized>(rng: &mut R) -> Self {
                    rng.gen::<$t>()
                }
            }
        )*
    };
}

impl_rand_construct_for_float!{f32, f64}

// Macro to implement RandConstruct for integer types
macro_rules! impl_rand_construct_for_integer {
    ($($t:ty),*) => {
        $(
            impl RandConstruct for $t {
                fn random() -> Self {
                    rand::random::<$t>()
                }
                fn uniform() -> Self {
                    let mut rng = rand::thread_rng();
                    rng.gen_range(<$t>::MIN..=<$t>::MAX)
                }
                fn random_with_rng<R: Rng + ?Sized>(rng: &mut R) -> Self {
                    rng.gen::<$t>()
                }
            }
        )*
    };
}

impl_rand_construct_for_integer!{i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize}
