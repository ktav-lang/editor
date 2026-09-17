//! Hover support: dotted-path lookup in the cached parse tree and
//! value description rendering.

use ktav::Value;

/// Walk a dotted path through a [`Value::Object`] tree.
pub(super) fn lookup_dotted<'a>(root: &'a Value, dotted: &str) -> Option<&'a Value> {
    let mut cur = root;
    for seg in dotted.split('.') {
        let Value::Object(map) = cur else { return None };
        cur = map.get(seg)?;
    }
    Some(cur)
}

pub(super) fn describe_value(v: &Value) -> String {
    match v {
        Value::Null => "null".into(),
        Value::Bool(b) => format!("bool: `{}`", b),
        Value::Integer(s) => format!("integer (typed): `{}`", s.as_str()),
        Value::Float(s) => format!("float (typed): `{}`", s.as_str()),
        Value::String(s) => {
            let shown = if s.len() > 80 {
                format!("{}…", &s[..80])
            } else {
                s.to_string()
            };
            format!("string: `{}`", shown)
        }
        Value::Array(a) => format!("array of {} items", a.len()),
        Value::Object(o) => format!("object with {} keys", o.len()),
    }
}
