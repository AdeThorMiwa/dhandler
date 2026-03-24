pub fn strip_code_block(s: &str) -> &str {
    if let Some(inner) = s
        .strip_prefix("```json")
        .and_then(|s| s.strip_suffix("```"))
    {
        inner.trim()
    } else if let Some(inner) = s.strip_prefix("```").and_then(|s| s.strip_suffix("```")) {
        inner.trim()
    } else {
        s
    }
}
