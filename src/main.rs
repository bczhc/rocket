#[macro_use]
extern crate rocket;

#[launch]
fn rocket() -> _ {
    rocket_demo::security::init();

    println!("Server started");
    use rocket_demo::routes;
    rocket::build().mount("/", routes![routes::text_transfer::text])
}
