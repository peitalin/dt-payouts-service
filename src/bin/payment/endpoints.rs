
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Endpoint<'a> {
    /// "Base" endpoint URL is set for:
    /// 1. GraphiQL Playground
    /// 2. Email verification and resets
    /// or any HTML templates that refer back to api endpoints
    /// from the end user's perspective.
    /// e.g: https://api.fileworks.net/user
    /// It is set with the ENDPOINT environment variable
    Base(&'a str),
    Payment(&'a str),
    User(&'a str),
    Content(&'a str),
    Upload(&'a str),
    Gateway(&'a str),
    Shopping(&'a str),
    Affiliate(&'a str),
}
impl<'a> Endpoint<'a> {
    pub fn as_url(&self) -> String {
        format!("{}", self)
    }
    pub fn as_path(&self) -> &str {
        match *self {
            Endpoint::Base(path) => path,
            Endpoint::Payment(path) => path,
            Endpoint::User(path) => path,
            Endpoint::Content(path) => path,
            Endpoint::Upload(path) => path,
            Endpoint::Gateway(path) => path,
            Endpoint::Shopping(path) => path,
            Endpoint::Affiliate(path) => path,
        }
    }
}
impl<'a> From<Endpoint<'a>> for String {
    fn from(e: Endpoint) -> String {
        e.as_url()
    }
}
impl<'a> std::fmt::Display for Endpoint<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        // e.g: 0.0.0.0:8082
        let endpoint = match *self {
            Endpoint::Base(path) => format_endpoint("ENDPOINT", path),
            Endpoint::Payment(path) => format_endpoint("EFC_PAYMENT_URL", path),
            Endpoint::User(path) => format_endpoint("EFC_USER_URL", path),
            Endpoint::Content(path) => format_endpoint("EFC_CONTENT_URL", path),
            Endpoint::Upload(path) => format_endpoint("EFC_UPLOAD_URL", path),
            Endpoint::Gateway(path) => format_endpoint("EFC_GATEWAY_URL", path),
            Endpoint::Shopping(path) => format_endpoint("EFC_SHOPPING_URL", path),
            Endpoint::Affiliate(path) => format_endpoint("EFC_AFFILIATE_URL", path),
        };
        write!(f, "{}", endpoint)
    }
}

fn format_endpoint(var: &str, path: &str) -> String {
    use std::env;
    dotenv::dotenv().ok();
    let uri = trim_trailing_slash(env::var(var))
            .expect(&format!("{} .env var to be set", var));
    format!("{}{}", uri, path)
}

pub fn check_env_var_endpoints_exist() {
    let _ = Endpoint::Base("/");
    let _ = Endpoint::User("/");
    let _ = Endpoint::Gateway("/");
    let _ = Endpoint::Shopping("/");
    let _ = Endpoint::Affiliate("/");
}

use std::env::VarError;
fn trim_trailing_slash(
    var: Result<String, VarError>
) -> Result<String, VarError> {
    match var {
        Err(e) => Err(e),
        Ok(mut s) => {
            if s.ends_with("/") {
                s.pop();
                Ok(s)
            } else {
                Ok(s)
            }
        }
    }
}

fn check_leading_slash(s: &str) -> String {
    if s.starts_with("/") {
        s.to_string()
    } else {
        format!("/{}", s)
    }
}