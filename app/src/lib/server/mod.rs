use rocket::{
    Build, Rocket,
    fairing::{AdHoc, Fairing, Info, Kind},
    http::Header,
};
pub mod browserhash;
#[derive(Database)]
#[database("primary")]
pub struct PrimaryDatabase(rocket_db_pools::sqlx::SqlitePool);

struct CspFairing;

#[rocket::async_trait]
impl Fairing for CspFairing {
    fn info(&self) -> Info {
        Info {
            name: "Add CSP Header",
            kind: Kind::Response,
        }
    }

    async fn on_response<'r>(
        &self,
        _request: &'r rocket::Request<'_>,
        response: &mut rocket::Response<'r>,
    ) {
        response.set_header(Header::new(
            "Content-Security-Policy",
            "default-src 'self'; frame-ancestors 'none'; script-src 'self'; connect-src 'self'; style-src 'self'; object-src 'none'",
        ));
    }
}




#[rocket::get("/")]
fn index() -> rocket_dyn_templates::Template {
    Template::render("index", rocket_dyn_templates::context! {})
}


use rocket_db_pools::Database;
use rocket_dyn_templates::Template;

// use crate::{CspFairing, PrimaryDatabase};
// #[launch]
pub fn rocket() -> Rocket<Build> {
    Rocket::build()
        .mount("/", rocket::routes![index])
        .mount(
            "/public",
            rocket::fs::FileServer::from(
                std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("public"),
            ),
        )
        // content security policy
        .attach(PrimaryDatabase::init())
        .attach(CspFairing)
        .attach(rocket_dyn_templates::Template::fairing())
}
