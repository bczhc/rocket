#[macro_use]
extern crate rocket;

#[launch]
fn rocket() -> _ {
    use rocket_demo::routes;
    rocket::build().mount("/", routes![routes::text])
}
