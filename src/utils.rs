pub fn language_from_locale() -> &'static str {
    let locale = std::env::var("LANG").unwrap_or_else(|_| "en_US".to_string());
    if locale.starts_with("es") {
        "es"
    } else if locale.starts_with("it") {
        "it"
    } else {
        "us"
    }
}
