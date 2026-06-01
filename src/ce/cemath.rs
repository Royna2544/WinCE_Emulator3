#[derive(Debug, Clone, PartialEq)]
pub enum CeMathCall {
    Abs(i32),
    Labs(i32),
    Div {
        numer: i32,
        denom: i32,
    },
    Ldiv {
        numer: i32,
        denom: i32,
    },
    UnaryF64 {
        op: CeMathUnaryF64,
        value: f64,
    },
    BinaryF64 {
        op: CeMathBinaryF64,
        lhs: f64,
        rhs: f64,
    },
    Frexp(f64),
    Ldexp {
        value: f64,
        exp: i32,
    },
    Modf(f64),
    LlMul {
        lhs: i64,
        rhs: i64,
    },
    LlDiv {
        lhs: i64,
        rhs: i64,
    },
    LlRem {
        lhs: i64,
        rhs: i64,
    },
    UllDiv {
        lhs: u64,
        rhs: u64,
    },
    UllRem {
        lhs: u64,
        rhs: u64,
    },
    LlLShift {
        value: i64,
        shift: u32,
    },
    LlRShift {
        value: i64,
        shift: u32,
    },
    UllRShift {
        value: u64,
        shift: u32,
    },
    FloatAdd {
        lhs: f32,
        rhs: f32,
    },
    FloatSub {
        lhs: f32,
        rhs: f32,
    },
    FloatMul {
        lhs: f32,
        rhs: f32,
    },
    FloatDiv {
        lhs: f32,
        rhs: f32,
    },
    DoubleAdd {
        lhs: f64,
        rhs: f64,
    },
    DoubleSub {
        lhs: f64,
        rhs: f64,
    },
    DoubleMul {
        lhs: f64,
        rhs: f64,
    },
    DoubleDiv {
        lhs: f64,
        rhs: f64,
    },
    FloatToLong(f32),
    DoubleToLong(f64),
    FloatToUnsignedLong(f32),
    DoubleToUnsignedLong(f64),
    LongToFloat(i32),
    LongToDouble(i32),
    UnsignedLongToFloat(u32),
    UnsignedLongToDouble(u32),
    FloatToDouble(f32),
    DoubleToFloat(f64),
    FloatCmp {
        lhs: f32,
        rhs: f32,
    },
    DoubleCmp {
        lhs: f64,
        rhs: f64,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CeMathUnaryF64 {
    Acos,
    Asin,
    Atan,
    Ceil,
    Cos,
    Cosh,
    Exp,
    Fabs,
    Floor,
    Log,
    Log10,
    Sin,
    Sinh,
    Sqrt,
    Tan,
    Tanh,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CeMathBinaryF64 {
    Atan2,
    Fmod,
    Pow,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CeMathValue {
    I32(i32),
    U32(u32),
    I64(i64),
    U64(u64),
    F32(f32),
    F64(f64),
    Div { quot: i32, rem: i32 },
    Frexp { fraction: f64, exp: i32 },
    Modf { integer: f64, fraction: f64 },
    Cmp(i32),
    DivideByZero,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct CeMath;

impl CeMath {
    pub fn eval(&self, call: CeMathCall) -> CeMathValue {
        match call {
            CeMathCall::Abs(value) | CeMathCall::Labs(value) => {
                CeMathValue::I32(value.saturating_abs())
            }
            CeMathCall::Div { numer, denom } | CeMathCall::Ldiv { numer, denom } => {
                if denom == 0 {
                    CeMathValue::DivideByZero
                } else {
                    CeMathValue::Div {
                        quot: numer / denom,
                        rem: numer % denom,
                    }
                }
            }
            CeMathCall::UnaryF64 { op, value } => CeMathValue::F64(eval_unary_f64(op, value)),
            CeMathCall::BinaryF64 { op, lhs, rhs } => {
                CeMathValue::F64(eval_binary_f64(op, lhs, rhs))
            }
            CeMathCall::Frexp(value) => {
                let (fraction, exp) = frexp(value);
                CeMathValue::Frexp { fraction, exp }
            }
            CeMathCall::Ldexp { value, exp } => CeMathValue::F64(value * 2.0_f64.powi(exp)),
            CeMathCall::Modf(value) => {
                let integer = value.trunc();
                CeMathValue::Modf {
                    integer,
                    fraction: value - integer,
                }
            }
            CeMathCall::LlMul { lhs, rhs } => CeMathValue::I64(lhs.wrapping_mul(rhs)),
            CeMathCall::LlDiv { lhs, rhs } => checked_i64_div(lhs, rhs),
            CeMathCall::LlRem { lhs, rhs } => checked_i64_rem(lhs, rhs),
            CeMathCall::UllDiv { lhs, rhs } => {
                if rhs == 0 {
                    CeMathValue::DivideByZero
                } else {
                    CeMathValue::U64(lhs / rhs)
                }
            }
            CeMathCall::UllRem { lhs, rhs } => {
                if rhs == 0 {
                    CeMathValue::DivideByZero
                } else {
                    CeMathValue::U64(lhs % rhs)
                }
            }
            CeMathCall::LlLShift { value, shift } => {
                CeMathValue::I64(value.wrapping_shl(shift.min(63)))
            }
            CeMathCall::LlRShift { value, shift } => CeMathValue::I64(value >> shift.min(63)),
            CeMathCall::UllRShift { value, shift } => CeMathValue::U64(value >> shift.min(63)),
            CeMathCall::FloatAdd { lhs, rhs } => CeMathValue::F32(lhs + rhs),
            CeMathCall::FloatSub { lhs, rhs } => CeMathValue::F32(lhs - rhs),
            CeMathCall::FloatMul { lhs, rhs } => CeMathValue::F32(lhs * rhs),
            CeMathCall::FloatDiv { lhs, rhs } => CeMathValue::F32(lhs / rhs),
            CeMathCall::DoubleAdd { lhs, rhs } => CeMathValue::F64(lhs + rhs),
            CeMathCall::DoubleSub { lhs, rhs } => CeMathValue::F64(lhs - rhs),
            CeMathCall::DoubleMul { lhs, rhs } => CeMathValue::F64(lhs * rhs),
            CeMathCall::DoubleDiv { lhs, rhs } => CeMathValue::F64(lhs / rhs),
            CeMathCall::FloatToLong(value) => CeMathValue::I32(value as i32),
            CeMathCall::DoubleToLong(value) => CeMathValue::I32(value as i32),
            CeMathCall::FloatToUnsignedLong(value) => CeMathValue::U32(value as u32),
            CeMathCall::DoubleToUnsignedLong(value) => CeMathValue::U32(value as u32),
            CeMathCall::LongToFloat(value) => CeMathValue::F32(value as f32),
            CeMathCall::LongToDouble(value) => CeMathValue::F64(value as f64),
            CeMathCall::UnsignedLongToFloat(value) => CeMathValue::F32(value as f32),
            CeMathCall::UnsignedLongToDouble(value) => CeMathValue::F64(value as f64),
            CeMathCall::FloatToDouble(value) => CeMathValue::F64(value as f64),
            CeMathCall::DoubleToFloat(value) => CeMathValue::F32(value as f32),
            CeMathCall::FloatCmp { lhs, rhs } => CeMathValue::Cmp(cmp_f32(lhs, rhs)),
            CeMathCall::DoubleCmp { lhs, rhs } => CeMathValue::Cmp(cmp_f64(lhs, rhs)),
        }
    }
}

impl CeMathCall {
    pub fn export_name(&self) -> &'static str {
        match self {
            Self::Abs(_) => "abs",
            Self::Labs(_) => "labs",
            Self::Div { .. } => "div",
            Self::Ldiv { .. } => "ldiv",
            Self::UnaryF64 { op, .. } => op.export_name(),
            Self::BinaryF64 { op, .. } => op.export_name(),
            Self::Frexp(_) => "frexp",
            Self::Ldexp { .. } => "ldexp",
            Self::Modf(_) => "modf",
            Self::LlMul { .. } => "__ll_mul",
            Self::LlDiv { .. } => "__ll_div",
            Self::LlRem { .. } => "__ll_rem",
            Self::UllDiv { .. } => "__ull_div",
            Self::UllRem { .. } => "__ull_rem",
            Self::LlLShift { .. } => "__ll_lshift",
            Self::LlRShift { .. } => "__ll_rshift",
            Self::UllRShift { .. } => "__ull_rshift",
            Self::FloatAdd { .. } => "__fpadd",
            Self::FloatSub { .. } => "__fpsub",
            Self::FloatMul { .. } => "__fpmul",
            Self::FloatDiv { .. } => "__fpdiv",
            Self::DoubleAdd { .. } => "__dpadd",
            Self::DoubleSub { .. } => "__dpsub",
            Self::DoubleMul { .. } => "__dpmul",
            Self::DoubleDiv { .. } => "__dpdiv",
            Self::FloatToLong(_) => "__fptoli",
            Self::FloatToUnsignedLong(_) => "__fptoul",
            Self::DoubleToLong(_) => "__dptoli",
            Self::DoubleToUnsignedLong(_) => "__dptoul",
            Self::LongToFloat(_) => "__litofp",
            Self::LongToDouble(_) => "__litodp",
            Self::UnsignedLongToFloat(_) => "__ultofp",
            Self::UnsignedLongToDouble(_) => "__ultodp",
            Self::FloatToDouble(_) => "__fptodp",
            Self::DoubleToFloat(_) => "__dptofp",
            Self::FloatCmp { .. } => "__fpcmp",
            Self::DoubleCmp { .. } => "__dpcmp",
        }
    }
}

impl CeMathUnaryF64 {
    fn export_name(self) -> &'static str {
        match self {
            Self::Acos => "acos",
            Self::Asin => "asin",
            Self::Atan => "atan",
            Self::Ceil => "ceil",
            Self::Cos => "cos",
            Self::Cosh => "cosh",
            Self::Exp => "exp",
            Self::Fabs => "fabs",
            Self::Floor => "floor",
            Self::Log => "log",
            Self::Log10 => "log10",
            Self::Sin => "sin",
            Self::Sinh => "sinh",
            Self::Sqrt => "sqrt",
            Self::Tan => "tan",
            Self::Tanh => "tanh",
        }
    }
}

impl CeMathBinaryF64 {
    fn export_name(self) -> &'static str {
        match self {
            Self::Atan2 => "atan2",
            Self::Fmod => "fmod",
            Self::Pow => "pow",
        }
    }
}

fn eval_unary_f64(op: CeMathUnaryF64, value: f64) -> f64 {
    match op {
        CeMathUnaryF64::Acos => value.acos(),
        CeMathUnaryF64::Asin => value.asin(),
        CeMathUnaryF64::Atan => value.atan(),
        CeMathUnaryF64::Ceil => value.ceil(),
        CeMathUnaryF64::Cos => value.cos(),
        CeMathUnaryF64::Cosh => value.cosh(),
        CeMathUnaryF64::Exp => value.exp(),
        CeMathUnaryF64::Fabs => value.abs(),
        CeMathUnaryF64::Floor => value.floor(),
        CeMathUnaryF64::Log => value.ln(),
        CeMathUnaryF64::Log10 => value.log10(),
        CeMathUnaryF64::Sin => value.sin(),
        CeMathUnaryF64::Sinh => value.sinh(),
        CeMathUnaryF64::Sqrt => value.sqrt(),
        CeMathUnaryF64::Tan => value.tan(),
        CeMathUnaryF64::Tanh => value.tanh(),
    }
}

fn eval_binary_f64(op: CeMathBinaryF64, lhs: f64, rhs: f64) -> f64 {
    match op {
        CeMathBinaryF64::Atan2 => lhs.atan2(rhs),
        CeMathBinaryF64::Fmod => lhs % rhs,
        CeMathBinaryF64::Pow => lhs.powf(rhs),
    }
}

fn frexp(value: f64) -> (f64, i32) {
    if value == 0.0 || !value.is_finite() {
        return (value, 0);
    }
    let exp = value.abs().log2().floor() as i32 + 1;
    (value / 2.0_f64.powi(exp), exp)
}

fn checked_i64_div(lhs: i64, rhs: i64) -> CeMathValue {
    if rhs == 0 {
        CeMathValue::DivideByZero
    } else {
        CeMathValue::I64(lhs.wrapping_div(rhs))
    }
}

fn checked_i64_rem(lhs: i64, rhs: i64) -> CeMathValue {
    if rhs == 0 {
        CeMathValue::DivideByZero
    } else {
        CeMathValue::I64(lhs.wrapping_rem(rhs))
    }
}

fn cmp_f32(lhs: f32, rhs: f32) -> i32 {
    lhs.partial_cmp(&rhs).map(ordering_to_i32).unwrap_or(1)
}

fn cmp_f64(lhs: f64, rhs: f64) -> i32 {
    lhs.partial_cmp(&rhs).map(ordering_to_i32).unwrap_or(1)
}

fn ordering_to_i32(ordering: std::cmp::Ordering) -> i32 {
    match ordering {
        std::cmp::Ordering::Less => -1,
        std::cmp::Ordering::Equal => 0,
        std::cmp::Ordering::Greater => 1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn evaluates_basic_crt_math() {
        let math = CeMath;
        assert_eq!(math.eval(CeMathCall::Abs(-7)), CeMathValue::I32(7));
        assert_eq!(
            math.eval(CeMathCall::Div {
                numer: 17,
                denom: 5,
            }),
            CeMathValue::Div { quot: 3, rem: 2 }
        );
        assert_eq!(
            math.eval(CeMathCall::BinaryF64 {
                op: CeMathBinaryF64::Pow,
                lhs: 2.0,
                rhs: 8.0,
            }),
            CeMathValue::F64(256.0)
        );
    }

    #[test]
    fn evaluates_mips_helpers() {
        let math = CeMath;
        assert_eq!(
            math.eval(CeMathCall::LlMul { lhs: 6, rhs: 7 }),
            CeMathValue::I64(42)
        );
        assert_eq!(
            math.eval(CeMathCall::DoubleCmp { lhs: 1.0, rhs: 2.0 }),
            CeMathValue::Cmp(-1)
        );
    }
}
