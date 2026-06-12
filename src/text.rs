pub fn line_safe(text: &str) -> String {
    text.chars()
        .map(|character| match character {
            '\r' | '\n' => ' ',
            character if character.is_control() => ' ',
            character => character,
        })
        .collect()
}

pub fn format_rate_per_hour(rate: f64) -> String {
    format!("{rate:.1}/h")
}
