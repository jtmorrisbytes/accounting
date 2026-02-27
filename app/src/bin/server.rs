use rocket::launch;
#[launch]
pub fn launch() -> _ {
   accounting::server::rocket()
}