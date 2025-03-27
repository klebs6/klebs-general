// ---------------- [ File: workspacer-cratesio-mock/src/bin/main.rs ]
use workspacer_cratesio_mock::*;
use workspacer_3p::*;

// We define a main function that starts rocket on e.g. `127.0.0.1:8888`
#[rocket::launch]
fn rocket_main() -> _ {

    let state = AppStateBuilder::default()
        .db(Arc::new(AsyncMutex::new(MockCratesDb::default())))
        .build()
        .unwrap();

    rocket::build()
        .manage(state)
        .mount("/", rocket::routes![publish_new])
        .register("/", rocket::catchers![not_found])
}
