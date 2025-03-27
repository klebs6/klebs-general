// ---------------- [ File: workspacer-cratesio-mock/src/bin/main.rs ]
use workspacer_cratesio_mock::*;
use workspacer_3p::*;

//use rocket::{Request, data::ToByteUnit, http::Status};
//use rocket::data::Data;
//use rocket::catch;
//use rocket::post;
//use rocket::serde::{Serialize, Deserialize, json::Json};
use std::sync::{Arc, Mutex};
//use std::collections::HashMap;
//use std::io::Read;


// We define a main function that starts rocket on e.g. `127.0.0.1:8888`
#[rocket::launch]
fn rocket_main() -> _ {

    let state = AppStateBuilder::default()
        .db(Arc::new(Mutex::new(MockCratesDb::default())))
        .build()
        .unwrap();

    rocket::build()
        .manage(state)
        .mount("/", rocket::routes![publish_new])
        .register("/", rocket::catchers![not_found])
}
