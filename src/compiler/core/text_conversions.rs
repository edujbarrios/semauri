// Shared by constant folding and evaluation for consistent validation.
fn parse_number_text(input: &str) -> Option<NumberValue> {
    let text = input.trim();
    if text.is_empty() { return None; }
    if let Ok(integer) = text.parse::<i64>() {
        return Some(NumberValue::Int(integer));
    }
    // Never silently round integer text outside the exact signed 64-bit range.
    if !text.contains('.') && !text.contains('e') && !text.contains('E') {
        return None;
    }
    let decimal = text.parse::<f64>().ok()?;
    if decimal.is_finite() { Some(NumberValue::Float(decimal)) } else { None }
}

fn parse_boolean_text(input: &str) -> Option<bool> {
    if input.trim().eq_ignore_ascii_case("true") { Some(true) }
    else if input.trim().eq_ignore_ascii_case("false") { Some(false) }
    else { None }
}
