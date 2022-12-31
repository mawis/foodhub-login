use crate::jwt::generate_jwt;
use crate::Config;
use log::{error, info};
use reqwest::StatusCode;
use rocket::form::{Form, FromForm};
use rocket::http::uri::Uri;
use rocket::http::{Cookie, CookieJar, SameSite};
use rocket::response::Redirect;
use rocket::{get, post, uri, State};
use rocket_include_tera::{tera_response, EtagIfNoneMatch, TeraContextManager, TeraResponse};
use serde::Deserialize;
use std::collections::HashMap;
use thiserror::Error;

#[derive(FromForm, Debug)]
pub struct LoginForm {
    privacy_policy: Option<String>,
}

#[derive(Deserialize)]
pub struct TokenResponse {
    access_token: String,
    token_type: String,
}

#[derive(Error, Debug)]
pub enum AuthServerError {
    #[error("reqwest error")]
    ReqwestError(#[from] reqwest::Error),

    #[error("HTTP state error")]
    HttpStateError { state: StatusCode, body: String },
}

#[get("/?<code>")]
pub async fn get_index_with_code(
    code: &str,
    tera_cm: &State<TeraContextManager>,
    etag_if_none_match: EtagIfNoneMatch<'_>,
    cookies: &CookieJar<'_>,
    config: &State<Config>,
) -> Result<Redirect, TeraResponse> {
    info!("Code is {:?}", code);

    let token = get_access_token(config, code).await;

    match token {
        Err(AuthServerError::HttpStateError { state, body }) => {
            let mut context = HashMap::new();
            context.insert("code", state.as_str().to_string());
            context.insert("body", body);
            Err(tera_response!(
                tera_cm,
                etag_if_none_match,
                "autherror",
                context
            ))
        }
        Err(AuthServerError::ReqwestError(_)) => {
            let mut context = HashMap::new();
            context.insert("code", String::new());
            context.insert("body", format!("{:?}", token));
            Err(tera_response!(
                tera_cm,
                etag_if_none_match,
                "autherror",
                context
            ))
        }
        Ok(token) => {
            let jwt = generate_jwt(&config.jwt_key);
            let auth_cookie = Cookie::build("x_auth_token", jwt.clone())
                .path("/")
                .secure(true)
                .http_only(true)
                .same_site(SameSite::Lax)
                .finish();
            cookies.add(auth_cookie);

            let mut context = HashMap::<&str, String>::new();
            context.insert("jwt", jwt);
            Err(tera_response!(
                tera_cm,
                etag_if_none_match,
                "loggedin",
                context
            ))
            //Ok(Redirect::to("/"))
        }
    }
}

#[get("/token?<token>")]
pub async fn get_token(token: String, cookies: &CookieJar<'_>) -> Redirect {
    let auth_cookie = Cookie::build("x_auth_token", token)
        .path("/")
        .secure(true)
        .http_only(true)
        .same_site(SameSite::Lax)
        .finish();
    cookies.add(auth_cookie);
    Redirect::to("/")
}

#[get("/", rank = 1)]
pub async fn get_index(
    tera_cm: &State<TeraContextManager>,
    etag_if_none_match: EtagIfNoneMatch<'_>,
) -> TeraResponse {
    let context = HashMap::<String, String>::new();
    tera_response!(tera_cm, etag_if_none_match, "login", context)
}

#[post("/", data = "<form>")]
pub async fn post_index(
    tera_cm: &State<TeraContextManager>,
    etag_if_none_match: EtagIfNoneMatch<'_>,
    form: Form<LoginForm>,
    cookies: &CookieJar<'_>,
    config: &State<Config>,
) -> Result<Redirect, TeraResponse> {
    match form.privacy_policy == Some("accepted".to_string()) {
        true => {
            let policy_accepted_cookie = Cookie::build("privacy_policy", "accepted")
                .path("/")
                .secure(true)
                .finish();
            cookies.add(policy_accepted_cookie);
            Ok(Redirect::to(start_oauth_uri(config)))
        }
        false => {
            let mut context = HashMap::<&str, &str>::new();
            context.insert("checkbox_class", "invalid-feedback");
            Err(tera_response!(
                tera_cm,
                etag_if_none_match,
                "login",
                context
            ))
        }
    }
}

async fn get_access_token(config: &Config, code: &str) -> Result<String, AuthServerError> {
    let client = reqwest::Client::builder()
        .cookie_store(false)
        .user_agent("Schichtvertretung")
        .build()?;
    let body = token_body(config, code);
    info!("Body: {}", body);
    let res = client
        .post(&config.token_endpoint)
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await?;

    let status = res.status();
    if status != StatusCode::OK {
        return Err(AuthServerError::HttpStateError {
            state: status,
            body: res.text().await?,
        });
    }

    //let token: TokenResponse = res.json().await?;

    Ok(String::from("got token!"))
}

fn start_oauth_uri(config: &Config) -> String {
    format!(
        "{}?response_type=code&client_id={}&redirect_uri={}&scope=openid&state=foo",
        &config.authorization_endpoint,
        &config.client_id,
        urlencoding::encode(&config.redirect_uri)
    )
}

fn token_body(config: &Config, code: &str) -> String {
    format!(
        "grant_type=authorization_code&code={}&client_id={}&redirect_uri={}&client_secret={}",
        code,
        &config.client_id,
        urlencoding::encode(&config.redirect_uri),
        &config.client_secret
    )
}
