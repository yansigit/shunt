/// Build the Chat Completions endpoint URL from the configured API root.
///
/// This is the single shared grammar for config boot validation and request
/// construction (CHAT-02/D-02): exactly one /chat/completions path, a
/// trailing-slash normalization, no doubling for roots that already end in
/// /chat/completions, and hard rejection of query strings, fragments, and
/// userinfo. Deterministic: repeated builds yield identical bytes.
pub fn chat_completions_endpoint(base_url: &str) -> Result<String, String> {
    if base_url
        .chars()
        .any(|c| c.is_whitespace() || c.is_control() || c == '\\')
    {
        return Err("base URL must not contain whitespace, controls, or backslashes".into());
    }
    let rest = base_url
        .strip_prefix("https://")
        .or_else(|| base_url.strip_prefix("http://"))
        .ok_or_else(|| "base URL must use an explicit http:// or https:// authority".to_string())?;
    let authority = rest.split('/').next().unwrap_or_default();
    if authority.is_empty() || authority.contains('@') {
        return Err("base URL requires a nonempty authority without userinfo".into());
    }
    if rest.split('/').skip(1).any(|segment| {
        matches!(
            segment.to_ascii_lowercase().as_str(),
            "." | ".." | "%2e" | ".%2e" | "%2e." | "%2e%2e"
        )
    }) {
        return Err("base URL must not contain dot path segments".into());
    }
    let url = reqwest::Url::parse(base_url).map_err(|_| "invalid base URL".to_string())?;
    if !matches!(url.scheme(), "http" | "https") {
        return Err("base URL must use http or https".into());
    }
    if url.query().is_some() {
        return Err("base URL must not contain a query string".into());
    }
    if url.fragment().is_some() {
        return Err("base URL must not contain a fragment".into());
    }
    if !url.username().is_empty() || url.password().is_some() {
        return Err("base URL must not contain userinfo (user:pass@)".into());
    }
    let host = url
        .host_str()
        .ok_or_else(|| "base URL must include a host".to_string())?;
    let authority = match url.port() {
        Some(port) => format!("{host}:{port}"),
        None => host.to_string(),
    };
    let path = url.path().trim_end_matches('/');
    let base = if path.is_empty() {
        format!("{}://{authority}", url.scheme())
    } else {
        format!("{}://{authority}{path}", url.scheme())
    };
    if base.ends_with("/chat/completions") {
        Ok(base)
    } else {
        Ok(format!("{base}/chat/completions"))
    }
}
