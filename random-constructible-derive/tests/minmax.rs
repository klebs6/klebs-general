use tracing::*;
use traced_test::*;
use tracing_setup::*;
use getset::*;
use derive_builder::*;
use random_constructible_derive::*;
use random_constructible::*;

#[derive(RandConstruct,Getters,Builder,Debug)]
#[getset(get="pub")]
#[builder(setter(into))]
struct MyClampedStruct {
    #[rand_construct(min=10, max=20)]
    my_number: i32,

    #[rand_construct(min=0.2, max=0.8)]
    fraction: f64,
}

#[traced_test]
fn verify_min_max_clamping() {
    info!("Starting test: verify_min_max_clamping for MyClampedStruct");

    let instance_random = MyClampedStruct::random();
    debug!("Got instance_random: {:?}", instance_random);

    // i32 clamp
    assert!(instance_random.my_number() >= &10);
    assert!(instance_random.my_number() <= &20);

    // f64 clamp
    assert!(instance_random.fraction() >= &0.2);
    assert!(instance_random.fraction() <= &0.8);

    let instance_uniform = MyClampedStruct::uniform();
    debug!("Got instance_uniform: {:?}", instance_uniform);

    // i32 clamp
    assert!(instance_uniform.my_number() >= &10);
    assert!(instance_uniform.my_number() <= &20);

    // f64 clamp
    assert!(instance_uniform.fraction() >= &0.2);
    assert!(instance_uniform.fraction() <= &0.8);
}

