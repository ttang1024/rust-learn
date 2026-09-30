use crate::checked::div;
use crate::checked::ln;
use crate::checked::sqrt;
use crate::checked::MathError;
use crate::checked::MathResult;

mod checked {
    // 我们想要捕获的数学 “错误”
    #[derive(Debug)]
    pub enum MathError {
        DivisionByZero,
        NegativeLogarithm,
        NegativeSquareRoot,
    }

    pub type MathResult = Result<f64, MathError>;

    pub fn div(x: f64, y: f64) -> MathResult {
        if y == 0.0 {
            // 此操作将会失败，那么（与其让程序崩溃）不如把失败的原因包装在
            // `Err` 中并返回
            Err(MathError::DivisionByZero)
        } else {
            // 此操作是有效的，返回包装在 `Ok` 中的结果
            Ok(x / y)
        }
    }

    pub fn sqrt(x: f64) -> MathResult {
        if x < 0.0 {
            Err(MathError::NegativeSquareRoot)
        } else {
            Ok(x.sqrt())
        }
    }

    pub fn ln(x: f64) -> MathResult {
        if x < 0.0 {
            Err(MathError::NegativeLogarithm)
        } else {
            Ok(x.ln())
        }
    }
}

// // `op(x, y)` === `sqrt(ln(x / y))`
// fn op(x: f64, y: f64) -> f64 {
//     // 这是一个三层的 match 金字塔！
//     match div(x, y) {
//         Err(why) => panic!("{:?}", why),
//         Ok(ratio) => match ln(ratio) {
//             Err(why) => panic!("{:?}", why),
//             Ok(ln) => match sqrt(ln) {
//                 Err(why) => panic!("{:?}", why),
//                 Ok(sqrt) => sqrt,
//             },
//         },
//     }
// }

// 中间函数
fn op_(x: f64, y: f64) -> MathResult {
    // 如果 `div` “失败” 了，那么返回 `DivisionByZero`
    let ratio = div(x, y)?;

    // 如果 `ln` “失败” 了，那么返回 `NegativeLogarithm`
    let ln = ln(ratio)?;

    sqrt(ln)
}

pub fn op(x: f64, y: f64) {
    match op_(x, y) {
        Err(why) => panic!(
            "{}",
            match why {
                MathError::NegativeLogarithm => "NegativeLogarithm",
                MathError::DivisionByZero => "DivisionByZero",
                MathError::NegativeSquareRoot => "NegativeSquareRoot",
            }
        ),
        Ok(value) => println!("{}", value),
    }
}

fn main() {
    println!("{:?}", op(1.0, 10.0));
}
