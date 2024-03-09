/// nvalue.rs

/// NValueKind
pub enum NValueKind {
    Empty,
    Int(i64),
    Float(f64),
    Bool(bool),
    String(String),
    Array(Vec<NValue>),
    Blob(Vec<u8>),
}

/// NValue
pub struct NValue {
    pub kind: NValueKind,
    pub tag: i64,
}

impl NValue {
    pub fn new() -> NValue {
        NValue { kind: NValueKind::Empty, tag: 0 }
    }
    pub fn empty() -> NValue {
        NValue::new()
    }
    pub fn from_int(v: i64) -> NValue {
        NValue { kind: NValueKind::Int(v), tag: 0 }
    }
    pub fn from_float(v: f64) -> NValue {
        NValue { kind: NValueKind::Float(v), tag: 0 }
    }
    pub fn from_str(v: &str) -> NValue {
        NValue { kind: NValueKind::String(String::from(v)), tag: 0 }
    }
    pub fn to_int_def(&self, def_value: i64) -> i64 {
        match &self.kind {
            NValueKind::Int(v) => *v,
            NValueKind::Float(v) => *v as i64,
            NValueKind::Bool(v) => if *v { 1 } else { 0 },
            NValueKind::String(v) => {
                match v.parse::<i64>() {
                    Ok(v) => v,
                    Err(_) => def_value,
                }
            },
            NValueKind::Array(_v) => def_value,
            _ => def_value,
        }
    }
    pub fn to_float_def(&self, def_value: f64) -> f64 {
        match &self.kind {
            NValueKind::Int(v) => *v as f64,
            NValueKind::Float(v) => *v,
            NValueKind::Bool(v) => if *v { 1.0 } else { 0.0 },
            NValueKind::String(v) => {
                match v.parse::<f64>() {
                    Ok(v) => v,
                    Err(_) => def_value,
                }
            },
            NValueKind::Array(_v) => def_value,
            _ => def_value,
        }
    }
    pub fn to_string(&self) -> String {
        match &self.kind {
            NValueKind::Int(v) => v.to_string(),
            NValueKind::Float(v) => v.to_string(),
            NValueKind::Bool(v) => v.to_string(),
            NValueKind::String(v) => v.clone(),
            NValueKind::Array(_v) => String::from("<Array>"),
            _ => String::from("<Empty>"),
        }
    }
    pub fn to_bool(&self) -> bool {
        match &self.kind {
            NValueKind::Int(v) => *v != 0,
            NValueKind::Float(v) => *v != 0.0,
            NValueKind::Bool(v) => *v,
            NValueKind::String(v) => !v.is_empty(),
            NValueKind::Array(v) => !v.is_empty(),
            _ => false,
        }
    }
    pub fn add_value(&self, value: NValue) -> NValue {
        match &self.kind {
            NValueKind::Int(v) => {
                match value.kind {
                    NValueKind::Int(v2) => NValue::from_int(*v + v2),
                    NValueKind::Float(v2) => NValue::from_float(*v as f64 + v2),
                    NValueKind::String(v2) => NValue::from_str(&format!("{}{}", v, v2)),
                    _ => NValue::empty(),
                }
            },
            NValueKind::Float(v) => {
                match value.kind {
                    NValueKind::Int(v2) => NValue::from_float(*v + v2 as f64),
                    NValueKind::Float(v2) => NValue::from_float(*v + v2),
                    NValueKind::String(v2) => NValue::from_str(&format!("{}{}", v, v2)),
                    _ => NValue::empty(),
                }
            },
            NValueKind::String(v) => {
                match value.kind {
                    NValueKind::Int(v2) => NValue::from_str(&format!("{}{}", v, v2)),
                    NValueKind::Float(v2) => NValue::from_str(&format!("{}{}", v, v2)),
                    NValueKind::String(v2) => NValue::from_str(&format!("{}{}", v, v2)),
                    _ => NValue::empty(),
                }
            },
            _ => NValue::empty(),
        }
    }
    pub fn mul_value(&self, value: NValue) -> NValue {
        match &self.kind {
            NValueKind::Int(v) => {
                match &value.kind {
                    NValueKind::Int(v2) => NValue::from_int(*v * v2),
                    NValueKind::Float(v2) => NValue::from_float(*v as f64 * v2),
                    NValueKind::String(_v2) => NValue::from_int(*v * value.to_int_def(0)),
                    _ => NValue::empty(),
                }
            },
            NValueKind::Float(v) => {
                match &value.kind {
                    NValueKind::Int(v2) => NValue::from_float(*v * (*v2 as f64)),
                    NValueKind::Float(v2) => NValue::from_float(*v * v2),
                    NValueKind::String(_v2) => NValue::from_float(*v * value.to_float_def(0.0)),
                    _ => NValue::empty(),
                }
            },
            NValueKind::String(v) => {
                match &value.kind {
                    NValueKind::Int(v2) => {
                        let mut s = String::new();
                        for _ in 0..*v2 {
                            s.push_str(v);
                        }
                        NValue::from_str(&s)
                    },
                    NValueKind::Float(v2) => {
                        let mut s = String::new();
                        for _ in 0..(*v2 as i64) {
                            s.push_str(v);
                        }
                        NValue::from_str(&s)
                    },
                    NValueKind::String(_v2) => {
                        let mut s = String::new();
                        for _ in 0..value.to_int_def(0) {
                            s.push_str(v);
                        }
                        NValue::from_str(&s)
                    },
                    _ => NValue::empty(),
                }
            },
            _ => NValue::empty(),
        }
    }
}

#[cfg(test)]
mod test_runner {
    use super::*;

    #[test]
    fn test_nvalue() {
        let v = NValue::from_int(10);
        assert_eq!(v.to_int_def(10), 10);
        let v = NValue::from_str("100");
        assert_eq!(v.to_int_def(0), 100);
        let a = NValue::from_str("ab");
        let b = a.mul_value(NValue::from_int(3));
        assert_eq!(b.to_string(), "ababab");
    }
}
