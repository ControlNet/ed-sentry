use toml::Value;

pub(super) fn read_string(
    value: Option<&Value>,
    key: &str,
    target: &mut String,
    warnings: &mut Vec<String>,
) {
    if let Some(value) = value {
        if let Some(parsed) = value.as_str() {
            *target = parsed.to_string();
        } else {
            warnings.push(wrong_type_warning(key));
        }
    }
}

pub(super) fn read_optional_string(
    value: Option<&Value>,
    key: &str,
    target: &mut Option<String>,
    warnings: &mut Vec<String>,
) {
    if let Some(value) = value {
        if let Some(parsed) = value.as_str() {
            *target = Some(parsed.to_string());
        } else {
            warnings.push(wrong_type_warning(key));
        }
    }
}

pub(super) fn read_bool(
    value: Option<&Value>,
    key: &str,
    target: &mut bool,
    warnings: &mut Vec<String>,
) {
    if let Some(value) = value {
        if let Some(parsed) = value.as_bool() {
            *target = parsed;
        } else {
            warnings.push(wrong_type_warning(key));
        }
    }
}

pub(super) fn read_u8(
    value: Option<&Value>,
    key: &str,
    target: &mut u8,
    warnings: &mut Vec<String>,
) {
    if let Some(value) = value {
        if let Some(parsed) = value
            .as_integer()
            .and_then(|number| u8::try_from(number).ok())
        {
            *target = parsed;
        } else {
            warnings.push(wrong_type_warning(key));
        }
    }
}

pub(super) fn read_u16(
    value: Option<&Value>,
    key: &str,
    target: &mut u16,
    warnings: &mut Vec<String>,
) {
    if let Some(value) = value {
        if let Some(parsed) = value
            .as_integer()
            .and_then(|number| u16::try_from(number).ok())
        {
            *target = parsed;
        } else {
            warnings.push(wrong_type_warning(key));
        }
    }
}

pub(super) fn read_u64(
    value: Option<&Value>,
    key: &str,
    target: &mut u64,
    warnings: &mut Vec<String>,
) {
    if let Some(value) = value {
        if let Some(parsed) = value
            .as_integer()
            .and_then(|number| u64::try_from(number).ok())
        {
            *target = parsed;
        } else {
            warnings.push(wrong_type_warning(key));
        }
    }
}

fn wrong_type_warning(key: &str) -> String {
    format!("config key {key} has wrong type; using default")
}
