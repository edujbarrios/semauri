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

