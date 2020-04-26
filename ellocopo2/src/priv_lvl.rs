
use num_enum::TryFromPrimitive;
use num_enum::IntoPrimitive;

#[repr(u8)]
#[allow(non_camel_case_types)]
#[derive(Clone, Copy, Debug, TryFromPrimitive, IntoPrimitive, PartialEq, Eq, PartialOrd, Ord)]
pub enum PrivLvl {
    NORMAL_LVL = 0,
    MODE1_LVL = 1,
    MODE2_LVL = 2,
    MODE3_LVL = 3,
    SECUR_LVL = 100,
    DEVEL_LVL = 254,
    UNDEF_LVL = 255,
}
