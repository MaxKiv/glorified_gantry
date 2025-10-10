/// CiA 402 standard homing methods (object 0x6098 " Homing Method ").
/// Each variant corresponds to a standard method code.
/// See CiA 402 Table 46 and Nanotec drive documentation.
#[repr(i8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HomingMethods {
    /// 0 — No homing operation defined
    None = 0,
    /// 1 — Move to negative limit switch, then to index pulse
    NegLimitThenIndex = 1,
    /// 2 — Move to positive limit switch, then to index pulse
    PosLimitThenIndex = 2,
    /// 17 — Move to positive limit switch only (no index)
    PosLimitOnly = 17,
    /// 18 — Move to negative limit switch only (no index)
    NegLimitOnly = 18,
    /// 33 — Home on positive switch, then index
    PosSwitchThenIndex = 33,
    /// 34 — Home on index pulse only (no switch)
    IndexOnly = 34,
    /// 35 — Set current position as home (no movement)
    CurrentPosition = 35,
}

impl HomingMethods {
    /// Convert to raw numeric value for SDO write (object 0x6098).
    #[inline]
    pub const fn as_i8(self) -> i8 {
        self as i8
    }

    /// Create a variant from a raw numeric value (SDO upload).
    pub const fn from_i8(value: i8) -> Option<Self> {
        match value {
            0 => Some(Self::None),
            1 => Some(Self::NegLimitThenIndex),
            2 => Some(Self::PosLimitThenIndex),
            17 => Some(Self::PosLimitOnly),
            18 => Some(Self::NegLimitOnly),
            33 => Some(Self::PosSwitchThenIndex),
            34 => Some(Self::IndexOnly),
            35 => Some(Self::CurrentPosition),
            _ => None,
        }
    }
}

impl From<HomingMethods> for i8 {
    fn from(method: HomingMethods) -> Self {
        method.as_i8()
    }
}

impl TryFrom<i8> for HomingMethods {
    type Error = ();
    fn try_from(value: i8) -> Result<Self, Self::Error> {
        Self::from_i8(value).ok_or(())
    }
}
