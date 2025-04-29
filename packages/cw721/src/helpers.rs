use cosmwasm_schema::cw_serde;
use cosmwasm_std::Addr;

/// Returns "empty" if the string is empty, otherwise the string itself
pub fn value_or_empty(value: &str) -> String {
    if value.is_empty() {
        "empty".to_string()
    } else {
        value.to_string()
    }
}

#[cw_serde]
pub struct Cw721Helper(pub Addr);
