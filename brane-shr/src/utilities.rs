use anyhow::Result;
use regex::Regex;
use url::Url;

///
///
///
pub fn ensure_http_schema<S>(
    url: S,
    secure: bool,
) -> Result<String>
where
    S: Into<String>,
{
    let url = url.into();
    let re = Regex::new(r"^https?://.*")?;

    let url = if re.is_match(&url) {
        url
    } else {
        format!("{}://{}", if secure { "https" } else { "http" }, url)
    };

    // Check if url is valid.
    let _ = Url::parse(&url)?;

    Ok(url)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ensurehttpschema_noschema_added() {
        let url = ensure_http_schema("localhost", true).unwrap();
        assert_eq!(url, "https://localhost");

        let url = ensure_http_schema("localhost", false).unwrap();
        assert_eq!(url, "http://localhost");
    }

    #[test]
    fn ensurehttpschema_schema_nothing() {
        let url = ensure_http_schema("http://localhost", true).unwrap();
        assert_eq!(url, "http://localhost");

        let url = ensure_http_schema("https://localhost", false).unwrap();
        assert_eq!(url, "https://localhost");
    } 
}
