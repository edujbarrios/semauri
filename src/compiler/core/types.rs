// Swap Unicode upper/lower case for each scalar. Case mappings can expand;
// uncased characters, digits, whitespace, and punctuation remain unchanged.
fn text_swapcase(value: &str) -> String {
    let mut output = String::with_capacity(value.len());
    for ch in value.chars() {
        if ch.is_lowercase() {
            output.extend(ch.to_uppercase());
        } else if ch.is_uppercase() {
            output.extend(ch.to_lowercase());
        } else {
            output.push(ch);
        }
    }
    output
}

// Titlecase whitespace-delimited words by uppercasing the first scalar
// and lowercasing all subsequent scalars. Whitespace is preserved verbatim.
fn text_titlecase(value: &str) -> String {
    let mut result = String::with_capacity(value.len());
    let mut at_word_start = true;
    for ch in value.chars() {
        if ch.is_whitespace() {
            result.push(ch);
            at_word_start = true;
        } else if at_word_start {
            result.extend(ch.to_uppercase());
            at_word_start = false;
        } else {
            result.extend(ch.to_lowercase());
        }
    }
    result
}

// Lowercase the first Unicode scalar without changing the remaining text.
fn text_uncapitalize(value: &str) -> String {
    let mut chars = value.chars();
    match chars.next() {
        Some(first) => first.to_lowercase().chain(chars).collect(),
        None => String::new(),
    }
}

// Capitalize the first Unicode scalar without altering the remainder.
// Unicode uppercasing may expand a scalar to multiple characters.
fn text_capitalize(value: &str) -> String {
    let mut chars = value.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().chain(chars).collect(),
        None => String::new(),
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum NumberValue {
    Int(i64),
    Float(f64),
}

impl NumberValue {
    fn as_f64(&self) -> f64 {
        match self {
            NumberValue::Int(value) => *value as f64,
            NumberValue::Float(value) => *value,
        }
    }

    fn is_zero(&self) -> bool {
        self.as_f64() == 0.0
    }

    fn is_positive(&self) -> bool {
        self.as_f64() > 0.0
    }

    fn is_negative(&self) -> bool {
        self.as_f64() < 0.0
    }

    fn is_integer(&self) -> bool {
        match self {
            NumberValue::Int(_) => true,
            NumberValue::Float(value) => value.fract() == 0.0,
        }
    }

    fn to_json(&self) -> JsonValue {
        match self {
            NumberValue::Int(value) => json!(value),
            NumberValue::Float(value) => json!(value),
        }
    }

    fn from_f64(value: f64) -> Self {
        if value.fract() == 0.0 && value >= i64::MIN as f64 && value <= i64::MAX as f64 {
            NumberValue::Int(value as i64)
        } else {
            NumberValue::Float(value)
        }
    }
}

// Pure numeric sign helper shared by constant folding and runtime evaluation.
// Signed zero maps to zero, and non-finite floating values are rejected.
fn numeric_sign(value: NumberValue) -> Option<NumberValue> {
    let sign = match value {
        NumberValue::Int(n) => n.signum(),
        NumberValue::Float(n) if n.is_finite() => {
            if n > 0.0 { 1 } else if n < 0.0 { -1 } else { 0 }
        }
        NumberValue::Float(_) => return None,
    };
    Some(NumberValue::Int(sign))
}

// Rounding preserves exact integers (including i64 extremes). Rounded
// floats become integers only inside the safely representable i64 range.
// In particular, f64(i64::MAX) equals 2^63, which does NOT fit in i64.
fn numeric_round(value: NumberValue, ceiling: bool) -> Option<NumberValue> {
    match value {
        NumberValue::Int(n) => Some(NumberValue::Int(n)),
        NumberValue::Float(n) if n.is_finite() => {
            let rounded = if ceiling { n.ceil() } else { n.floor() };
            if rounded >= i64::MIN as f64 && rounded < -(i64::MIN as f64) {
                Some(NumberValue::Int(rounded as i64))
            } else {
                Some(NumberValue::Float(rounded))
            }
        }
        NumberValue::Float(_) => None,
    }
}

// Round to nearest integer; halfway cases go away from zero. Exact signed
// integers stay exact. A rounded floating value outside the signed i64 range
// remains a float instead of saturating to i64::MIN/MAX.
fn numeric_nearest(value: NumberValue) -> Option<NumberValue> {
    match value {
        NumberValue::Int(n) => Some(NumberValue::Int(n)),
        NumberValue::Float(n) if n.is_finite() => {
            let rounded = n.round();
            if rounded >= i64::MIN as f64 && rounded < -(i64::MIN as f64) {
                Some(NumberValue::Int(rounded as i64))
            } else {
                Some(NumberValue::Float(rounded))
            }
        }
        NumberValue::Float(_) => None,
    }
}

// Truncate the fractional part toward zero. Preserve exact i64 integers,
// and only cast finite floats that safely fit the signed 64-bit range.
fn numeric_truncate(value: NumberValue) -> Option<NumberValue> {
    match value {
        NumberValue::Int(n) => Some(NumberValue::Int(n)),
        NumberValue::Float(n) if n.is_finite() => {
            let truncated = n.trunc();
            if truncated >= i64::MIN as f64 && truncated < -(i64::MIN as f64) {
                Some(NumberValue::Int(truncated as i64))
            } else {
                Some(NumberValue::Float(truncated))
            }
        }
        NumberValue::Float(_) => None,
    }
}

// A checked, bounded factorial. 20! fits i64 while 21! does not.
// Integral decimal operands are accepted only in the same finite domain.
fn numeric_factorial(value: NumberValue) -> Option<NumberValue> {
    let n = match value {
        NumberValue::Int(n) if (0..=20).contains(&n) => n as u32,
        NumberValue::Float(n) if n.is_finite() && n.fract() == 0.0
            && (0.0..=20.0).contains(&n) => n as u32,
        _ => return None,
    };
    let mut result: i64 = 1;
    for i in 2..=n {
        result = result.checked_mul(i as i64)?;
    }
    Some(NumberValue::Int(result))
}

// Deterministic modular exponentiation for exact 64-bit prime checks.
fn prime_mod_pow(mut base: u64, mut exponent: u64, modulus: u64) -> u64 {
    let mut answer = 1_u64;
    while exponent != 0 {
        if exponent & 1 == 1 {
            answer = ((answer as u128 * base as u128) % modulus as u128) as u64;
        }
        base = ((base as u128 * base as u128) % modulus as u128) as u64;
        exponent >>= 1;
    }
    answer
}

// Deterministic Miller-Rabin witnesses for all unsigned 64-bit integers.
// Signed Semauri numbers are a strict subset of that domain.
fn numeric_prime(value: NumberValue) -> Option<bool> {
    let signed = match value {
        NumberValue::Int(n) => n,
        NumberValue::Float(n) if n.is_finite() && n.fract() == 0.0
            && n >= i64::MIN as f64 && n < -(i64::MIN as f64) => n as i64,
        _ => return None,
    };
    if signed < 2 { return Some(false); }
    let n = signed as u64;
    for divisor in [2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37] {
        if n == divisor { return Some(true); }
        if n % divisor == 0 { return Some(false); }
    }
    let power_of_two = (n - 1).trailing_zeros();
    let d = (n - 1) >> power_of_two;
    for witness in [2_u64, 325, 9375, 28178, 450775, 9780504, 1795265022] {
        let a = witness % n;
        if a == 0 { continue; }
        let mut x = prime_mod_pow(a, d, n);
        if x == 1 || x == n - 1 { continue; }
        let mut passed = false;
        for _ in 1..power_of_two {
            x = ((x as u128 * x as u128) % n as u128) as u64;
            if x == n - 1 { passed = true; break; }
        }
        if !passed { return Some(false); }
    }
    Some(true)
}

// Exact signed integer cubing; float multiplication is finite-checked.
fn numeric_cube(value: NumberValue) -> Option<NumberValue> {
    match value {
        NumberValue::Int(n) => n.checked_mul(n)?.checked_mul(n).map(NumberValue::Int),
        NumberValue::Float(n) if n.is_finite() => {
            let result = n * n * n;
            if result.is_finite() { Some(NumberValue::Float(result)) } else { None }
        }
        NumberValue::Float(_) => None,
    }
}

// Shared by constant folding and evaluation: checked signed 64-bit
// integer multiplication and finite floating-point results only.
fn numeric_square(value: NumberValue) -> Option<NumberValue> {
    match value {
        NumberValue::Int(n) => n.checked_mul(n).map(NumberValue::Int),
        NumberValue::Float(n) if n.is_finite() => {
            let result = n * n;
            if result.is_finite() { Some(NumberValue::Float(result)) } else { None }
        }
        NumberValue::Float(_) => None,
    }
}

// Count base-10 digits of the magnitude of a signed 64-bit integer.
// The sign is not a digit; zero has one digit. Use unsigned_abs to
// support i64::MIN without signed overflow or floating-point rounding.
fn numeric_digits(value: NumberValue) -> Option<NumberValue> {
    let integer = match value {
        NumberValue::Int(n) => n,
        NumberValue::Float(n)
            if n.is_finite() && n.fract() == 0.0
                && n >= i64::MIN as f64 && n < -(i64::MIN as f64) => n as i64,
        _ => return None,
    };
    let magnitude = integer.unsigned_abs();
    Some(NumberValue::Int(if magnitude == 0 {
        1
    } else {
        (magnitude.ilog10() + 1) as i64
    }))
}

// Reciprocal is a floating-point operation. Reject zero, non-finite
// inputs, and any output that overflows to infinity.
fn numeric_reciprocal(value: NumberValue) -> Option<NumberValue> {
    let number = match value {
        NumberValue::Int(n) => n as f64,
        NumberValue::Float(n) => n,
    };
    if number == 0.0 || !number.is_finite() {
        return None;
    }
    let inverse = 1.0 / number;
    inverse.is_finite().then_some(NumberValue::Float(inverse))
}

// Perfect integer cube roots are located using exact u128 arithmetic.
// In particular, i64::MIN = (-2097152)^3 must stay an exact integer.
// Non-perfect integer inputs and finite floating-point inputs yield floats.
fn numeric_cbrt(value: NumberValue) -> Option<NumberValue> {
    match value {
        NumberValue::Int(n) => {
            let magnitude = u128::from(n.unsigned_abs());
            let (mut low, mut high) = (0_u64, 2_097_152_u64);
            while low <= high {
                let mid = low + (high - low) / 2;
                let cube = u128::from(mid).pow(3);
                if cube == magnitude {
                    let root = mid as i64;
                    return Some(NumberValue::Int(if n < 0 { -root } else { root }));
                }
                if cube < magnitude {
                    low = mid + 1;
                } else if mid == 0 {
                    break;
                } else {
                    high = mid - 1;
                }
            }
            Some(NumberValue::Float((n as f64).cbrt()))
        }
        NumberValue::Float(n) if n.is_finite() => Some(NumberValue::Float(n.cbrt())),
        NumberValue::Float(_) => None,
    }
}

// Square roots preserve perfect integer squares without relying on the
// rounded f64 representation of a large i64. Non-perfect roots are floats.
fn numeric_sqrt(value: NumberValue) -> Option<NumberValue> {
    match value {
        NumberValue::Int(n) if n >= 0 => {
            let root = (n as f64).sqrt();
            let candidate = root as u64;
            for possible in [candidate, candidate.saturating_add(1)] {
                let exact = (possible as u128) * (possible as u128);
                if exact == n as u128 {
                    return Some(NumberValue::Int(possible as i64));
                }
            }
            Some(NumberValue::Float(root))
        }
        NumberValue::Float(n) if n.is_finite() && n >= 0.0 => {
            let root = n.sqrt();
            if root.fract() == 0.0
                && root >= i64::MIN as f64 && root < -(i64::MIN as f64) {
                Some(NumberValue::Int(root as i64))
            } else {
                Some(NumberValue::Float(root))
            }
        }
        _ => None,
    }
}

// Finite scientific functions are real-valued and never produce NaN/Infinity.
fn numeric_transcendental(value: NumberValue, operator: UnaryOperator) -> Option<NumberValue> {
    let input = value.as_f64();
    if !input.is_finite() { return None; }
    let result = match operator {
        UnaryOperator::Exponential => input.exp(),
        UnaryOperator::NaturalLog if input > 0.0 => input.ln(),
        UnaryOperator::Sine => input.sin(),
        UnaryOperator::Cosine => input.cos(),
        _ => return None,
    };
    result.is_finite().then_some(NumberValue::Float(result))
}

// Parity accepts exact integers and integral finite decimals only when they
// fit signed i64. Rejecting out-of-range floats avoids saturating casts and
// meaningless parity tests above the precision range of binary64.
fn numeric_is_even(value: NumberValue) -> Option<bool> {
    match value {
        NumberValue::Int(n) => Some(n % 2 == 0),
        NumberValue::Float(n) if n.is_finite() && n.fract() == 0.0
            && n >= i64::MIN as f64 && n < -(i64::MIN as f64) =>
            Some((n as i64) % 2 == 0),
        _ => None,
    }
}

impl fmt::Display for NumberValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NumberValue::Int(value) => write!(f, "{value}"),
            NumberValue::Float(value) => {
                let mut rendered = value.to_string();
                if !rendered.contains('.') && !rendered.contains('e') && !rendered.contains('E') {
                    rendered.push_str(".0");
                }
                write!(f, "{rendered}")
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct NominalType {
    pub domain: String,
    pub name: String,
    pub base_type: Box<Type>,
    pub promote_from_base: bool,
}

impl NominalType {
    pub fn new(domain: &str, name: &str, base_type: Type, promote_from_base: bool) -> Self {
        Self {
            domain: domain.to_string(),
            name: name.to_string(),
            base_type: Box::new(base_type),
            promote_from_base,
        }
    }

    fn to_json(&self) -> JsonValue {
        json!({
            "kind": "nominal",
            "domain": self.domain,
            "name": self.name,
            "base": self.base_type.to_json(),
            "promote_from_base": self.promote_from_base
        })
    }
}

impl fmt::Display for NominalType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}", self.domain, self.name)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Type {
    Unit,
    Number,
    Boolean,
    String,
    Color,
    EntityRef,
    Opaque,
    List(Box<Type>),
    Nominal(NominalType),
}

impl Type {
    fn to_json(&self) -> JsonValue {
        match self {
            Type::Unit => json!("unit"),
            Type::Number => json!("number"),
            Type::Boolean => json!("boolean"),
            Type::String => json!("string"),
            Type::Color => json!("color"),
            Type::EntityRef => json!("entity_ref"),
            Type::Opaque => json!("opaque"),
            Type::List(element) => json!({
                "kind": "list",
                "element_type": element.to_json()
            }),
            Type::Nominal(nominal) => nominal.to_json(),
        }
    }

    fn assignment_kind(actual: &Type, expected: &Type) -> Option<AssignmentKind> {
        if actual == expected {
            return Some(AssignmentKind::Exact);
        }
        if let Type::Nominal(nominal) = expected {
            if nominal.promote_from_base && actual == nominal.base_type.as_ref() {
                return Some(AssignmentKind::Promote);
            }
        }
        None
    }
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Type::Unit => write!(f, "unit"),
            Type::Number => write!(f, "number"),
            Type::Boolean => write!(f, "boolean"),
            Type::String => write!(f, "string"),
            Type::Color => write!(f, "color"),
            Type::EntityRef => write!(f, "entity_ref"),
            Type::Opaque => write!(f, "opaque"),
            Type::List(element) => write!(f, "list<{element}>"),
            Type::Nominal(nominal) => write!(f, "{nominal}"),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum AssignmentKind {
    Exact,
    Promote,
}

#[derive(Clone, Debug, PartialEq)]
pub enum LiteralValue {
    Number(NumberValue),
    Boolean(bool),
    String(String),
    Color(String),
}

impl LiteralValue {
    fn to_json(&self) -> JsonValue {
        match self {
            LiteralValue::Number(value) => value.to_json(),
            LiteralValue::Boolean(value) => json!(value),
            LiteralValue::String(value) | LiteralValue::Color(value) => json!(value),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum ValueData {
    Number(NumberValue),
    Boolean(bool),
    String(String),
    Color(String),
    List(Vec<ValueData>),
    Unit,
}

impl ValueData {
    fn to_json(&self) -> JsonValue {
        match self {
            ValueData::Number(value) => value.to_json(),
            ValueData::Boolean(value) => json!(value),
            ValueData::String(value) | ValueData::Color(value) => json!(value),
            ValueData::List(values) => JsonValue::Array(values.iter().map(ValueData::to_json).collect()),
            ValueData::Unit => JsonValue::Null,
        }
    }

    fn describe(&self, ty: &Type) -> String {
        match self {
            ValueData::Number(value) => format!("{ty} {value}"),
            ValueData::Boolean(value) => format!("{ty} {value}"),
            ValueData::String(value) | ValueData::Color(value) => {
                format!("{ty} {:?}", value)
            }
            ValueData::List(values) => format!("{ty} {:?}", values.iter().map(ValueData::to_json).collect::<Vec<_>>()),
            ValueData::Unit => "unit nil".to_string(),
        }
    }
}

