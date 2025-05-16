use url::Url;

pub(crate) fn sanitize_ws_url(url: &str) -> String {
    let parsed = Url::parse(url);
    match parsed {
        Ok(mut u) => {
            let mut segments: Vec<_> = u.path_segments().map(|c| c.collect()).unwrap_or_default();

            if !segments.is_empty() {
                segments.pop(); // remove the last segment (e.g., a secret token)
                segments.push("xxxxx...");
                let sanitized_path = segments.join("/");
                u.set_path(&sanitized_path);
            }

            u.to_string()
        }
        Err(_) => "invalid_url".to_string(),
    }
}
