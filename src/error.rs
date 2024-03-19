use cosmwasm_std::{StdError, Timestamp};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Staked Token Is Burned By Admin")]
    StakedTokenIsBurnedByAdmin {},

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Unbounding at(will_finish: {will_finish})")]
    StillUnbounding { will_finish: Timestamp },

    #[error("Already unstaked")]
    AlreadyUnstaked {},

    #[error("Not whitelisted collection")]
    NotWhitelisted {},

    #[error("Already a whitelisted collection")]
    AlreadyWhitelisted {},
}