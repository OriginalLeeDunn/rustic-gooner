#[macro_use] extern crate rocket;

#[get("/")]
fn index() -> &'static str {
    "Eyyy, Voxel Rocket!"
}

#[launch]
fn rocket() -> _ {
    rocket::build().mount("/", routes![index])
}
