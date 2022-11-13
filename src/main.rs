#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_migrations;

use rocket::fairing::AdHoc;
use rocket::{routes, Build, Rocket};
use rocket_cors::CorsOptions;
use rocket_include_tera::{tera_resources_initialize, TeraResponse};
use rocket_sync_db_pools::database;
use uuid::Uuid;

mod web;

#[database("postgres_diesel")]
pub struct DbConn(diesel::PgConnection);

pub fn uuid_convert(ruuid: rocket::serde::uuid::Uuid) -> Uuid {
    Uuid::from_bytes(ruuid.into_bytes())
}

embed_migrations!();

async fn run_migrations(rocket: Rocket<Build>) -> Rocket<Build> {
    let conn = DbConn::get_one(&rocket)
        .await
        .expect("Could not get database connection");
    conn.run(|c| embedded_migrations::run(c))
        .await
        .expect("Could not run migrations");
    rocket
}

fn rocket() -> Rocket<Build> {
    rocket::build()
        .attach(DbConn::fairing())
        .attach(AdHoc::on_ignite("Diesel Migrations", run_migrations))
        .attach(CorsOptions::default().to_cors().unwrap())
        .attach(TeraResponse::fairing(|tera| {
            tera_resources_initialize!(
                tera,
                "base" => "views/base.tera",
                "login" => "views/login.tera"
            )
        }))
        .mount("/login", routes![web::login::get_index])
}

#[rocket::main]
async fn main() {
    let _ = rocket().launch().await.expect("Could not start rocket");
}
