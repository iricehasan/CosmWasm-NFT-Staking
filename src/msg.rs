use cosmwasm_schema::{cw_serde, QueryResponses};
use cw721::Cw721ReceiveMsg;
use crate::state::Staking;

#[cw_serde]
pub struct InstantiateMsg {
    pub admin: Option<String>,
    pub nft_addr: String,
}

#[cw_serde]
pub enum ExecuteMsg {
    ReceiveNft(Cw721ReceiveMsg),
    Unstake {
        index: u64,
    },
    Claim {
        index: u64,
    },
    AdminBurn {
        index: u64,
    },
    AddCollection { 
        nft_addr: String
    },
    RemoveCollection {
        nft_addr: String,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(WhitelistedNftAddressesResponse)]
    WhitelistedNftAddresses {},
    #[returns(AdminAddressResponse)]
    AdminAddress {},
    #[returns(StakingsResponse)]
    StakingsByAddress { address: String },
}

#[cw_serde]
pub struct WhitelistedNftAddressesResponse {
    pub nft_addrs: Vec<String>
}

#[cw_serde]
pub struct AdminAddressResponse {
    pub admin: String,
}

#[cw_serde]
pub struct StakingsResponse {
    pub stakings: Vec<Staking>,
}