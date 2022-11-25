#[macro_use]
extern crate rocket;

#[get("/")]
fn index() -> &'static str {
	"test"
}

#[launch]
fn rocket() -> _ {
	rocket::build().mount("/api/", routes![index])
}
