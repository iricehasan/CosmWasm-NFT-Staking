# NFT Staking Functionality

1. Staking contract is instantiated by setting an admin address and a nft_address as a whitelisted collection to be used in the contract.
2. Sending tokens from the whitelisted collections triggers the staking function.
3. Unstaking function triggers the unbonding period of 14 days, and only the position owner can trigger. After 14 days, claim function can be called by the user which transfers the token back to the user.
4. Admin of the staking contract can burn the NFT in staked position.
5. User cannot unstake if the NFT is burned by the admin.
6. Admin can add or remove collections to the whitelisted NFT addresses.

# Messages

```rust
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
```