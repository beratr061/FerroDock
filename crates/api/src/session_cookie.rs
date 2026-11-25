use axum_extra::extract::cookie::{Cookie, SameSite};
use time::OffsetDateTime;

const SECURE_NAME: &str = "__Host-fd_session";
const PLAIN_NAME: &str = "fd_session";

pub fn name(secure: bool) -> &'static str {
    if secure { SECURE_NAME } else { PLAIN_NAME }
}

fn base(value: String, secure: bool) -> Cookie<'static> {
    let mut cookie = Cookie::new(name(secure), value);
    cookie.set_http_only(true);
    cookie.set_secure(secure);
    cookie.set_same_site(SameSite::Lax);
    cookie.set_path("/");
    cookie
}

pub fn issue(token: &str, expires_at: OffsetDateTime, secure: bool) -> Cookie<'static> {
    let mut cookie = base(token.to_owned(), secure);
    cookie.set_expires(expires_at);
    cookie
}

pub fn clear(secure: bool) -> Cookie<'static> {
    let mut cookie = base(String::new(), secure);
    cookie.set_expires(OffsetDateTime::UNIX_EPOCH);
    cookie
}
