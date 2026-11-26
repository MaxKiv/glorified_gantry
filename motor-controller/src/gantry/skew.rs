use uom::si::{f64::Length, length::millimeter};

#[derive(Debug, Clone, PartialEq)]
pub struct GantrySkew {
    pub x: Option<Length>,
    pub y: Option<Length>,
    pub z: Option<Length>,
}

impl GantrySkew {
    pub fn exceeds(&self, val: Length) -> bool {
        self.x.is_some_and(|skew| skew > val)
            | self.y.is_some_and(|skew| skew > val)
            | self.z.is_some_and(|skew| skew > val)
    }
}

impl std::fmt::Display for GantrySkew {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "skew({:?}, {:?}, {:?})",
            self.x.map(|skew| skew.get::<millimeter>()),
            self.y.map(|skew| skew.get::<millimeter>()),
            self.z.map(|skew| skew.get::<millimeter>()),
        )
    }
}
