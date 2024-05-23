use actix_web::HttpRequest;

pub fn check_auth(rtoken: String, req: &HttpRequest) -> bool {
    let token = req
        .headers()
        .get("Authorization");
    if token.is_none() {
        return false;
    }
    let token = token.unwrap()
    .to_str()
    .unwrap()
    .trim_start_matches("Basic ");
    return rtoken == token;
}
