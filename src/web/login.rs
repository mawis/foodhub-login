use rocket::{get, State};
use rocket_include_tera::{tera_response, EtagIfNoneMatch, TeraContextManager, TeraResponse};
use std::collections::HashMap;

#[get("/")]
pub async fn get_index(
    tera_cm: &State<TeraContextManager>,
    etag_if_none_match: EtagIfNoneMatch<'_>,
) -> TeraResponse {
    let context = HashMap::<String, String>::new();
    tera_response!(tera_cm, etag_if_none_match, "login", context)
}
