pub fn cloudflare_trycloudflare_url(input: &str) -> Option<String> {
    let mut remaining = input;
    while let Some(index) = remaining.find("https://") {
        let candidate_start = &remaining[index..];
        let end = candidate_start
            .find(|character: char| {
                character.is_whitespace() || matches!(character, '"' | '\'' | '<' | '>')
            })
            .unwrap_or(candidate_start.len());
        let candidate = candidate_start[..end].trim_end_matches(&[',', '.', ')', ']'][..]);
        if public_host(candidate).is_some() {
            return Some(candidate.to_string());
        }
        remaining = &candidate_start["https://".len()..];
    }
    None
}

pub(super) fn public_host(public_url: &str) -> Option<String> {
    let without_scheme = public_url.strip_prefix("https://")?;
    let host_end = without_scheme
        .find(['/', '?', '#', ':'])
        .unwrap_or(without_scheme.len());
    let host = without_scheme[..host_end].to_ascii_lowercase();
    let suffix = ".trycloudflare.com";
    if host.len() > suffix.len() && host.ends_with(suffix) {
        return Some(host);
    }
    None
}
