use cosmwasm_std::{Addr, BankMsg, Coin, CosmosMsg, Response};

pub fn add_bank_msg(response: &mut Response, addr: &Addr, amount: u128, denom: &str) {
    // BankMsg fails if amount == 0
    if amount > 0 {
        response.messages.push(CosmosMsg::Bank(BankMsg::Send {
            to_address: addr.to_string(),
            amount: vec![Coin::new(amount, denom)],
        }));
    }
}
