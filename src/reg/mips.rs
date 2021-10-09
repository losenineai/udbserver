use crate::reg::RegMap;
use unicorn::RegisterMIPS;

pub static REGMAP: RegMap = RegMap {
    regs: &[
        (Some(RegisterMIPS::ZERO as i32), 4),
        (Some(RegisterMIPS::AT as i32), 4),
        (Some(RegisterMIPS::V0 as i32), 4),
        (Some(RegisterMIPS::V1 as i32), 4),
        (Some(RegisterMIPS::A0 as i32), 4),
        (Some(RegisterMIPS::A1 as i32), 4),
        (Some(RegisterMIPS::A2 as i32), 4),
        (Some(RegisterMIPS::A3 as i32), 4),
        (Some(RegisterMIPS::T0 as i32), 4),
        (Some(RegisterMIPS::T1 as i32), 4),
        (Some(RegisterMIPS::T2 as i32), 4),
        (Some(RegisterMIPS::T3 as i32), 4),
        (Some(RegisterMIPS::T4 as i32), 4),
        (Some(RegisterMIPS::T5 as i32), 4),
        (Some(RegisterMIPS::T6 as i32), 4),
        (Some(RegisterMIPS::T7 as i32), 4),
        (Some(RegisterMIPS::S0 as i32), 4),
        (Some(RegisterMIPS::S1 as i32), 4),
        (Some(RegisterMIPS::S2 as i32), 4),
        (Some(RegisterMIPS::S3 as i32), 4),
        (Some(RegisterMIPS::S4 as i32), 4),
        (Some(RegisterMIPS::S5 as i32), 4),
        (Some(RegisterMIPS::S6 as i32), 4),
        (Some(RegisterMIPS::S7 as i32), 4),
        (Some(RegisterMIPS::T8 as i32), 4),
        (Some(RegisterMIPS::T9 as i32), 4),
        (Some(RegisterMIPS::K0 as i32), 4),
        (Some(RegisterMIPS::K1 as i32), 4),
        (Some(RegisterMIPS::GP as i32), 4),
        (Some(RegisterMIPS::SP as i32), 4),
        (Some(RegisterMIPS::S8 as i32), 4),
        (Some(RegisterMIPS::RA as i32), 4),
        (None, 4), // status
        (Some(RegisterMIPS::LO as i32), 4),
        (Some(RegisterMIPS::HI as i32), 4),
        (None, 4), // badvaddr
        (None, 4), // cause
        (Some(RegisterMIPS::PC as i32), 4),
        (Some(RegisterMIPS::F0 as i32), 4),
        (Some(RegisterMIPS::F1 as i32), 4),
        (Some(RegisterMIPS::F2 as i32), 4),
        (Some(RegisterMIPS::F3 as i32), 4),
        (Some(RegisterMIPS::F4 as i32), 4),
        (Some(RegisterMIPS::F5 as i32), 4),
        (Some(RegisterMIPS::F6 as i32), 4),
        (Some(RegisterMIPS::F7 as i32), 4),
        (Some(RegisterMIPS::F8 as i32), 4),
        (Some(RegisterMIPS::F9 as i32), 4),
        (Some(RegisterMIPS::F10 as i32), 4),
        (Some(RegisterMIPS::F11 as i32), 4),
        (Some(RegisterMIPS::F12 as i32), 4),
        (Some(RegisterMIPS::F13 as i32), 4),
        (Some(RegisterMIPS::F14 as i32), 4),
        (Some(RegisterMIPS::F15 as i32), 4),
        (Some(RegisterMIPS::F16 as i32), 4),
        (Some(RegisterMIPS::F17 as i32), 4),
        (Some(RegisterMIPS::F18 as i32), 4),
        (Some(RegisterMIPS::F19 as i32), 4),
        (Some(RegisterMIPS::F20 as i32), 4),
        (Some(RegisterMIPS::F21 as i32), 4),
        (Some(RegisterMIPS::F22 as i32), 4),
        (Some(RegisterMIPS::F23 as i32), 4),
        (Some(RegisterMIPS::F24 as i32), 4),
        (Some(RegisterMIPS::F25 as i32), 4),
        (Some(RegisterMIPS::F26 as i32), 4),
        (Some(RegisterMIPS::F27 as i32), 4),
        (Some(RegisterMIPS::F28 as i32), 4),
        (Some(RegisterMIPS::F29 as i32), 4),
        (Some(RegisterMIPS::F30 as i32), 4),
        (Some(RegisterMIPS::F31 as i32), 4),
        (None, 4), // fcsr
        (None, 4), // fir
        (Some(RegisterMIPS::HI1 as i32), 4),
        (Some(RegisterMIPS::LO1 as i32), 4),
        (Some(RegisterMIPS::HI2 as i32), 4),
        (Some(RegisterMIPS::LO2 as i32), 4),
        (Some(RegisterMIPS::HI3 as i32), 4),
        (Some(RegisterMIPS::LO3 as i32), 4),
    ],
    len: 32,
    desc: r#"<target version="1.0"><architecture>mips</architecture></target>"#,
};
