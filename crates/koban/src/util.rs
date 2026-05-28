pub fn redact(input: impl Into<String>, token: &str) -> String {
    let input = input.into();
    if token.trim().is_empty() {
        input
    } else {
        input.replace(token, "[REDACTED]")
    }
}
