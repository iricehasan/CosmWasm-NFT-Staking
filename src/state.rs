use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Timestamp};
use cw_storage_plus::{Item, Map};

#[cw_serde]
pub struct Config {
    pub admin: Addr,
    pub whitelisted_nft_addrs: Vec<Addr>,
}

#[cw_serde]
pub struct Staking {
    pub nft_addr: String, 
    pub token_id: String,
    pub sender: String,
    pub start_timestamp: Timestamp,
    pub end_timestamp: Timestamp,
    pub is_burned_by_admin: bool,
}

pub const CONFIG: Item<Config> = Item::new("config");
pub const STAKINGS: Map<String, Vec<Staking>> = Map::new("stakings");