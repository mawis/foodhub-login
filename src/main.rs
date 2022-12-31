use rocket::fairing::AdHoc;
use rocket::{routes, Build, Rocket};
use rocket_cors::CorsOptions;
use rocket_include_tera::{tera_resources_initialize, TeraResponse};
use serde_derive::Deserialize;
use uuid::Uuid;

mod jwt;
mod web;

#[derive(Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct Config {
    authorization_endpoint: String,
    token_endpoint: String,
    client_id: String,
    client_secret: String,
    redirect_uri: String,
    jwt_key: String,
}

pub fn uuid_convert(ruuid: rocket::serde::uuid::Uuid) -> Uuid {
    Uuid::from_bytes(ruuid.into_bytes())
}

fn rocket() -> Rocket<Build> {
    rocket::build()
        .attach(AdHoc::config::<Config>())
        .attach(CorsOptions::default().to_cors().unwrap())
        .attach(TeraResponse::fairing(|tera| {
            tera_resources_initialize!(
                tera,
                "base" => "views/base.tera",
                "login" => "views/login.tera",
                "loggedin" => "views/loggedin.tera",
                "autherror" => "views/autherror.tera"
            )
        }))
        .mount(
            "/login",
            routes![
                web::login::get_index,
                web::login::post_index,
                web::login::get_index_with_code,
                web::login::get_token
            ],
        )
}

#[rocket::main]
async fn main() {
    let _ = rocket().launch().await.expect("Could not start rocket");
}
