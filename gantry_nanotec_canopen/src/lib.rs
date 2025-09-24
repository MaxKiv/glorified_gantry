pub mod error;
pub mod motor;
pub mod od;
pub mod pdo;

pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use crate::pdo::mapping::calculate_pdo_index_offset;

    use super::*;

    #[test]
    fn test_index_offset_calculation() {
        let result = calculate_pdo_index_offset(0x500, 3);
        assert_eq!(result, 0x503);

        let result = calculate_pdo_index_offset(0x180, 1);
        assert_eq!(result, 0x181);
    }
}
