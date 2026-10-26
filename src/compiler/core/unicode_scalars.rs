// Pure, bounded Unicode scalar conversion shared by evaluation and folding.
// Character positions and values are Unicode scalar values, not UTF-8 bytes
// or extended grapheme clusters. Surrogates and out-of-range values are invalid.
const MAX_UNICODE_SCALAR_LIST: usize = 10_000;

fn eval_unicode_scalar(operator: &UnaryOperator, value: &ValueData) -> Option<ValueData> {
    match (operator, value) {
        (UnaryOperator::CharacterFrom, ValueData::Number(NumberValue::Int(number))) => {
            let codepoint = u32::try_from(*number).ok()?;
            Some(ValueData::String(char::from_u32(codepoint)?.to_string()))
        }
        (UnaryOperator::Codepoint, ValueData::String(text)) => {
            let mut chars = text.chars();
            let one = chars.next()?;
            if chars.next().is_some() { return None; }
            Some(ValueData::Number(NumberValue::Int(one as i64)))
        }
        (UnaryOperator::Characters, ValueData::String(text)) => {
            let values: Vec<_> = text.chars().take(MAX_UNICODE_SCALAR_LIST + 1)
                .map(|ch| ValueData::String(ch.to_string())).collect();
            (values.len() <= MAX_UNICODE_SCALAR_LIST).then_some(ValueData::List(values))
        }
        (UnaryOperator::Codepoints, ValueData::String(text)) => {
            let values: Vec<_> = text.chars().take(MAX_UNICODE_SCALAR_LIST + 1)
                .map(|ch| ValueData::Number(NumberValue::Int(ch as i64))).collect();
            (values.len() <= MAX_UNICODE_SCALAR_LIST).then_some(ValueData::List(values))
        }
        _ => None,
    }
}
