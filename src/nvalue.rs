/// nvalue.rs

/// NValue
#[derive(Debug,Clone,PartialEq)]
pub enum NValue {
    Empty,
    NaN, // Not a Number (for calculation error)
    Int(i64),
    Float(f64),
    Bool(bool),
    String(String),
    Array(Vec<NValue>),
    Blob(Vec<u8>),
}

impl NValue {
    /// new NValue
    pub fn new() -> NValue {
        NValue::Empty
    }
    /// return Empty value
    pub fn empty() -> NValue {
        NValue::new()
    }
    /// return new Int value
    pub fn from_int(v: i64) -> NValue {
        NValue::Int(v)
    }
    /// return new Float value
    pub fn from_float(v: f64) -> NValue {
        NValue::Float(v)
    }
    /// return new String value
    pub fn from_str(v: &str) -> NValue {
        NValue::String(String::from(v))
    }
    /// return new String value
    pub fn from_string(v: String) -> NValue {
        NValue::String(v)
    }
    /// return new String value
    pub fn from_char(v: char) -> NValue {
        NValue::String(String::from(v))
    }
    /// check if value is empty
    pub fn is_empty(&self) -> bool {
        match &self {
            NValue::Empty => true,
            _ => false,
        }
    }
    /// convert to int
    pub fn to_int(&self) -> Result<i64, &str> {
        match &self {
            NValue::Int(v) => Ok(*v),
            NValue::Float(v) => Ok(*v as i64),
            NValue::Bool(v) => Ok(if *v { 1 } else { 0 }),
            NValue::String(v) => {
                if v.starts_with("0x") { // hex value
                    match i64::from_str_radix(&v[2..], 16) {
                        Ok(v) => return Ok(v),
                        Err(_) => return Err("string could not convert to int"),
                    }
                }
                else if v.starts_with("0o") { // oct value
                    match i64::from_str_radix(&v[2..], 8) {
                        Ok(v) => return Ok(v),
                        Err(_) => return Err("string could not convert to int"),
                    }
                }
                else if v.starts_with("0b") { // binary value
                    match i64::from_str_radix(&v[2..], 2) {
                        Ok(v) => return Ok(v),
                        Err(_) => return Err("string could not convert to int"),
                    }
                }
                match v.parse::<i64>() { // decimal value
                    Ok(v) => Ok(v),
                    Err(_) => Err("string could not convert to int"),
                }
            },
            NValue::Array(_v) => Err("array could not convert to int"),
            _ => Err("could not convert to int"),
        }
    }
    /// convert to int with default value
    pub fn to_int_def(&self, def_value: i64) -> i64 {
        match &self.to_int() {
            Ok(v) => *v,
            Err(_) => def_value,
        }
    }
    /// convert to float
    pub fn to_float(&self) -> Result<f64, &str> {
        match &self {
            NValue::Int(v) => Ok(*v as f64),
            NValue::Float(v) => Ok(*v),
            NValue::Bool(v) => Ok(if *v { 1.0 } else { 0.0 }),
            NValue::String(v) => {
                if v.starts_with("0x") || v.starts_with("0o") || v.starts_with("0b") {
                    match &self.to_int() {
                        Ok(int_value) => return Ok(*int_value as f64),
                        Err(_) => return Err("Cannot convert to float"),
                    }
                }
                match v.parse::<f64>() {
                    Ok(v) => Ok(v),
                    Err(_) => Err("Cannot convert to float"),
                }
            },
            NValue::Array(_v) => Err("Cannot convert to float"),
            _ => Err("Cannot convert to float"),
        }
    }
    /// convert to float with default value
    pub fn to_float_def(&self, def_value: f64) -> f64 {
        match &self.to_float() {
            Ok(v) => *v,
            Err(_) => def_value,
        }
    }
    /// convert to string
    pub fn to_string(&self) -> String {
        match &self {
            NValue::Int(v) => v.to_string(),
            NValue::Float(v) => v.to_string(),
            NValue::Bool(v) => v.to_string(),
            NValue::String(v) => v.clone(),
            NValue::Array(_v) => String::from("<Array>"),
            _ => String::from("<Empty>"),
        }
    }
    /// convert to bool
    pub fn to_bool(&self) -> bool {
        match &self {
            NValue::Int(v) => *v != 0,
            NValue::Float(v) => *v != 0.0,
            NValue::Bool(v) => *v,
            NValue::String(v) => {
                if v.eq("true") || v.eq("1") || v.eq("真") {
                    return true
                }
                if v.eq("false") || v.eq("0") || v.eq("偽") {
                    return false
                }
                !v.is_empty()
            },
            NValue::Array(v) => !v.is_empty(),
            _ => false,
        }
    }
    /// add value
    pub fn add_value(&self, value: NValue) -> NValue {
        match &self {
            NValue::Int(v) => {
                match value {
                    NValue::Int(v2) => NValue::from_int(*v + v2),
                    NValue::Float(v2) => NValue::from_float(*v as f64 + v2),
                    NValue::String(v2) => NValue::from_str(&format!("{}{}", v, v2)),
                    _ => NValue::NaN,
                }
            },
            NValue::Float(v) => {
                match value {
                    NValue::Int(v2) => NValue::from_float(*v + v2 as f64),
                    NValue::Float(v2) => NValue::from_float(*v + v2),
                    NValue::String(v2) => NValue::from_str(&format!("{}{}", v, v2)),
                    _ => NValue::NaN,
                }
            },
            NValue::String(v) => {
                match value {
                    NValue::Int(v2) => NValue::from_str(&format!("{}{}", v, v2)),
                    NValue::Float(v2) => NValue::from_str(&format!("{}{}", v, v2)),
                    NValue::String(v2) => NValue::from_str(&format!("{}{}", v, v2)),
                    _ => NValue::NaN,
                }
            },
            _ => NValue::empty(),
        }
    }
    /// mul value
    pub fn mul_value(&self, value: NValue) -> NValue {
        match &self {
            NValue::Int(v) => {
                match &value {
                    NValue::Int(v2) => NValue::from_int(*v * v2),
                    NValue::Float(v2) => NValue::from_float(*v as f64 * v2),
                    NValue::String(_v2) => NValue::from_int(*v * value.to_int_def(0)),
                    _ => NValue::NaN,
                }
            },
            NValue::Float(v) => {
                match &value {
                    NValue::Int(v2) => NValue::from_float(*v * (*v2 as f64)),
                    NValue::Float(v2) => NValue::from_float(*v * v2),
                    NValue::String(_v2) => NValue::from_float(*v * value.to_float_def(0.0)),
                    _ => NValue::NaN,
                }
            },
            NValue::String(v) => {
                match &value {
                    NValue::Int(v2) => {
                        let mut s = String::new();
                        for _ in 0..*v2 {
                            s.push_str(v);
                        }
                        NValue::from_str(&s)
                    },
                    NValue::Float(v2) => {
                        let mut s = String::new();
                        for _ in 0..(*v2 as i64) {
                            s.push_str(v);
                        }
                        NValue::from_str(&s)
                    },
                    NValue::String(_v2) => {
                        let mut s = String::new();
                        for _ in 0..value.to_int_def(0) {
                            s.push_str(v);
                        }
                        NValue::from_str(&s)
                    },
                    _ => NValue::NaN,
                }
            },
            _ => NValue::NaN,
        }
    }
    /// div value (return float value)
    pub fn div_value(&self, value: NValue) -> NValue {
        match &self {
            NValue::Int(v) => {
                match &value {
                    NValue::Int(v2) => NValue::from_float(*v as f64 / *v2 as f64),
                    NValue::Float(v2) => NValue::from_float(*v as f64 / v2),
                    NValue::String(_) => {
                        match value.to_float() {
                            Ok(v2) => NValue::from_float(*v as f64 / v2),
                            Err(_) => NValue::NaN,
                        }
                    }
                    _ => NValue::NaN,
                }
            },
            NValue::Float(v) => {
                match &value {
                    NValue::Int(v2) => NValue::from_float(*v / (*v2 as f64)),
                    NValue::Float(v2) => NValue::from_float(*v / v2),
                    NValue::String(_v2) => NValue::from_float(*v * value.to_float_def(0.0)),
                    _ => NValue::NaN,
                }
            },
            NValue::String(_) => {
                let av = match self.to_float() {
                    Ok(v) => v,
                    Err(_) => return NValue::NaN,
                };
                match &value {
                    NValue::Int(v2) => NValue::from_float(av / (*v2 as f64)),
                    NValue::Float(v2) => NValue::from_float(av / *v2),
                    NValue::String(_v2) => {
                        match value.to_float() {
                            Ok(v2) => NValue::from_float(av / v2),
                            Err(_) => NValue::NaN,
                        }
                    },
                    _ => NValue::NaN,
                }
            },
            _ => NValue::NaN,
        }
    }
    /// div value (return int value)
    pub fn div_int_value(&self, value: NValue) -> NValue {
        match &self {
            NValue::Int(v) => {
                match &value {
                    NValue::Int(v2) => NValue::from_int(*v / *v2),
                    NValue::Float(v2) => NValue::from_int(*v / *v2 as i64),
                    NValue::String(_) => {
                        match value.to_int() {
                            Ok(v2) => NValue::from_int(*v / v2),
                            Err(_) => NValue::NaN,
                        }
                    }
                    _ => NValue::NaN,
                }
            },
            NValue::Float(v) => {
                match &value {
                    NValue::Int(v2) => NValue::from_int(*v as i64 / *v2),
                    NValue::Float(v2) => NValue::from_int(*v as i64 / *v2 as i64),
                    NValue::String(_v2) => NValue::from_int(*v as i64 / value.to_int_def(0)),
                    _ => NValue::NaN,
                }
            },
            NValue::String(_) => {
                let av = match self.to_int() {
                    Ok(v) => v,
                    Err(_) => return NValue::NaN,
                };
                match &value {
                    NValue::Int(v2) => NValue::from_int(av / *v2),
                    NValue::Float(v2) => NValue::from_int(av / *v2 as i64),
                    NValue::String(_v2) => {
                        match value.to_int() {
                            Ok(v2) => NValue::from_int(av / v2),
                            Err(_) => NValue::NaN,
                        }
                    },
                    _ => NValue::NaN,
                }
            },
            _ => NValue::NaN,
        }
    }
    /// mod value (return float value)
    pub fn mod_value(&self, value: NValue) -> NValue {
        match &self {
            NValue::Int(v) => {
                match &value {
                    NValue::Int(v2) => NValue::from_int(*v as i64 % *v2 as i64),
                    NValue::Float(v2) => NValue::from_float(*v as f64 % *v2 as f64),
                    NValue::String(_) => {
                        match value.to_int() {
                            Ok(v2) => NValue::from_int(*v as i64 % v2),
                            Err(_) => NValue::NaN,
                        }
                    }
                    _ => NValue::NaN,
                }
            },
            NValue::Float(v) => {
                match &value {
                    NValue::Int(v2) => NValue::from_int(*v as i64 % *v2),
                    NValue::Float(v2) => NValue::from_float(*v % v2),
                    NValue::String(_v2) => NValue::from_float(*v % value.to_float_def(0.0)),
                    _ => NValue::NaN,
                }
            },
            NValue::String(_) => {
                let av = match self.to_int() {
                    Ok(v) => v,
                    Err(_) => return NValue::NaN,
                };
                match &value {
                    NValue::Int(v2) => NValue::from_int(av % v2),
                    NValue::Float(v2) => NValue::from_float(av as f64 % *v2),
                    NValue::String(_v2) => {
                        match value.to_int() {
                            Ok(v2) => NValue::from_int(av % v2),
                            Err(_) => NValue::NaN,
                        }
                    },
                    _ => NValue::NaN,
                }
            },
            _ => NValue::NaN,
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
    #[test]
    fn test_convert() {
        // str to int
        let a = NValue::from_str("30").to_int_def(0);
        assert_eq!(a, 30);
        // str to float
        let a = NValue::from_str("3.14").to_float_def(0.0);
        assert_eq!(a, 3.14);
        // str to bool
        let a = NValue::from_str("0").to_bool();
        assert_eq!(a, false);
        let a = NValue::from_str("1").to_bool();
        assert_eq!(a, true);
        let a = NValue::from_str("-1").to_bool();
        assert_eq!(a, true);
    }
    #[test]
    fn test_calc() {
        // int add int
        let a = NValue::from_int(30);
        let b = NValue::from_int(50);
        let c = a.add_value(b);
        assert_eq!(c.to_int_def(0), 80);
        // str add str
        let a = NValue::from_str("30");
        let b = NValue::from_str("20");
        let c = a.add_value(b);
        assert_eq!(c.to_int_def(0), 3020);
        // int mul int
        let a = NValue::from_int(30);
        let b = NValue::from_int(3);
        let c = a.mul_value(b);
        assert_eq!(c.to_int_def(0), 90);
        // str mul int
        let a = NValue::from_str("30");
        let b = NValue::from_int(3);
        let c = a.mul_value(b);
        assert_eq!(c.to_string(), String::from("303030"));
    }
    #[test]
    fn test_calc_div() {
        // int div int
        let a = NValue::from_int(30);
        let b = NValue::from_int(3);
        let c = a.div_value(b);
        assert_eq!(c.to_float_def(0.0), 10.0);
        // float div float
        let a = NValue::from_float(30.0);
        let b = NValue::from_float(3.0);
        let c = a.div_value(b);
        assert_eq!(c.to_float_def(0.0), 10.0);
        // float div 0
        let a = NValue::from_float(30.0);
        let b = NValue::from_float(0.0);
        let c = a.div_value(b);
        assert_eq!(c.to_float_def(0.0), f64::INFINITY);
        // int div_int
        let a = NValue::from_int(30);
        let b = NValue::from_int(5);
        let c = a.div_int_value(b);
        assert_eq!(c.to_int_def(0), 6);
        // int div_int 3
        let a = NValue::from_int(10);
        let b = NValue::from_int(3);
        let c = a.div_int_value(b);
        assert_eq!(c.to_int_def(0), 3);
        // str div_int 3
        let a = NValue::from_str("30");
        let b = NValue::from_int(3);
        let c = a.div_int_value(b);
        assert_eq!(c.to_int_def(0), 10);
        // str div_int str
        let a = NValue::from_str("30");
        let b = NValue::from_str("10");
        let c = a.div_int_value(b);
        assert_eq!(c.to_int_def(0), 3);
    }
}
