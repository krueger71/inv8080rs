//! Utilities

/// Get bit
/// ```
/// # use inv8080rs::utils::get_bit;
/// let data: u8 = 0b10100101;
/// 
/// assert!(get_bit(data, 0));
/// assert!(!get_bit(data, 1));
/// assert!(get_bit(data, 2));
/// assert!(!get_bit(data, 3));
/// assert!(!get_bit(data, 4));
/// assert!(get_bit(data, 5));
/// assert!(!get_bit(data, 6));
/// assert!(get_bit(data, 7));
/// ```

pub fn get_bit(val: u8, n: u8) -> bool {
    (val & (1 << n)) != 0
}

/// Set bit
/// ```
/// # use inv8080rs::utils::{set_bit, get_bit};
/// let mut data: u8 = 0;
/// 
/// for i in 0..8 {
///     set_bit(&mut data, i, true);
///     assert!(get_bit(data, i));
///     set_bit(&mut data, i, false);
///     assert!(!get_bit(data, i));
/// }
/// ```
pub fn set_bit(value: &mut u8, n: u8, val: bool) {
    if val {
        *value |= 1 << n;
    } else {
        *value &= !(1 << n);
    }
}
