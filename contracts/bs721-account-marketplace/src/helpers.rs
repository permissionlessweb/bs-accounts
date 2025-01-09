use cosmwasm_std::Uint128;

// Calculate the renewal price based on the name length
pub fn get_char_price(base_price: u128, name_len: usize) -> Uint128 {
    match name_len {
        0..=2 => unreachable!("name_len should be at least 3"),
        3 => base_price * 100,
        4 => base_price * 10,
        _ => base_price,
    }
    .into()
}
