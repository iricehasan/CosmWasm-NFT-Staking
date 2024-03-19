use std::vec;

#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, WasmMsg, Timestamp, Event, CosmosMsg
};

use cw721::{Cw721ExecuteMsg, Cw721ReceiveMsg};

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg, AdminAddressResponse, StakingsResponse, WhitelistedNftAddressesResponse};
use crate::state::{Config, CONFIG, Staking, STAKINGS};

pub const UNBONDING_PERIOD: u64 = 14 * 24 * 60 * 60; // 14 days unbonding period

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let admin = msg
        .admin
        .and_then(|addr_string| deps.api.addr_validate(addr_string.as_str()).ok())
        .unwrap_or(info.sender);
    
    let mut nft_addrs = vec![];
    let nft_addr = deps.api.addr_validate(&msg.nft_addr)?;
    nft_addrs.push(nft_addr);

    let config = Config {
        admin: admin.clone(),
        whitelisted_nft_addrs: nft_addrs,

    };

    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new()
        .add_attribute("action", "instantiate")
    )
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {

    match msg {
        ExecuteMsg::ReceiveNft(receive_msg) => execute_stake(deps, env, info, receive_msg),
        ExecuteMsg::Unstake { index } => execute_unstake(deps,env,info, index),
        ExecuteMsg::AdminBurn { index } => execute_admin_burn(deps,env,info, index),
        ExecuteMsg::AddCollection { nft_addr } => execute_add_collection(deps,env,info, nft_addr),
        ExecuteMsg::RemoveCollection { nft_addr } => execute_remove_collection(deps, env, info, nft_addr),
        ExecuteMsg::Claim { index } => execute_claim(deps, env, info, index),
    }
}

pub fn execute_stake(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    receive_msg: Cw721ReceiveMsg,
) -> Result<Response, ContractError> {

    // info.sender is the NFT contract Address
    let sender = receive_msg.sender.clone();
    let nft_addr = info.sender.clone().to_string();
    let token_id = receive_msg.token_id.clone();

    // check if the nft_addr is whitelisted
    let config = CONFIG.load(deps.storage)?;

    let nft_addr = deps.api.addr_validate(&nft_addr)?;

    if !config.whitelisted_nft_addrs.contains(&nft_addr) {
        return Err(ContractError::AlreadyWhitelisted {});
    }

    let stakings = STAKINGS.may_load(deps.storage, sender.clone())?;
    let mut stakings_state: Vec<Staking>;
    if stakings.is_none() {
        stakings_state = vec![]
    } else {
        stakings_state = stakings.unwrap()
    };
    stakings_state.push(Staking {
        nft_addr: nft_addr.clone().into_string(),
        sender: receive_msg.clone().sender,
        token_id: receive_msg.clone().token_id,
        start_timestamp: env.block.time,
        end_timestamp: Timestamp::from_seconds(0), // will be updated when unstake function is called
        is_burned_by_admin: false,
    });

    STAKINGS.save(deps.storage, sender.clone(), &stakings_state)?;

    Ok(Response::new().add_event(
        Event::new("staked")
            .add_attribute("nft_address", nft_addr)
            .add_attribute("token_id", token_id)
            .add_attribute("sender", sender)
            .add_attribute("start_timestamp", env.block.time.to_string())
    ))
}

// starts the unbonding
pub fn execute_unstake(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    index: u64,
) -> Result<Response, ContractError> {
    let mut stakings_state = STAKINGS.may_load(deps.storage, info.sender.to_string().clone())?.unwrap();
    let staking_info = stakings_state[index as usize].clone();
    let mut staking = &mut stakings_state[index as usize];

    // check if staking.nft_addr is in Config nft_addrs

    if staking.end_timestamp != Timestamp::from_nanos(0) {
        return Err(ContractError::AlreadyUnstaked {});
    }

    if info.sender != staking.sender {
        return Err(ContractError::Unauthorized {})
    }

    if staking.is_burned_by_admin {
        return Err(ContractError::StakedTokenIsBurnedByAdmin {})
    }

    staking.end_timestamp = env.block.time.plus_seconds(UNBONDING_PERIOD);

    let _ = STAKINGS.save(deps.storage, staking_info.sender.clone(), &stakings_state);

    Ok(Response::new()
        .add_event(
            Event::new("unstaked")
                .add_attribute("nft_addr", staking_info.nft_addr.clone())
                .add_attribute("token_id", staking_info.token_id.clone())
                .add_attribute("sender", staking_info.sender.clone())
                .add_attribute(
                    "start_timestamp",
                    staking_info.start_timestamp.seconds().to_string(),
                )
                .add_attribute("end_timestamp", staking_info.end_timestamp.seconds().to_string())
                .add_attribute("index", index.to_string()),
        ))
}

pub fn execute_admin_burn(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo, 
    index: u64,
) -> Result<Response, ContractError> {
    let config = CONFIG.load(deps.storage)?;

    let mut stakings_state = STAKINGS.may_load(deps.storage, info.sender.to_string().clone())?.unwrap();
    let staking_info = stakings_state[index as usize].clone();
    let mut staking = &mut stakings_state[index as usize];

    if info.sender != config.admin {
        return Err(ContractError::Unauthorized {})
    }

    // admin can burn the staked NFT
    let burn_msg: CosmosMsg = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: staking.nft_addr.clone(),
        msg: to_json_binary(&Cw721ExecuteMsg::Burn {
            token_id: staking.token_id.clone(),
        })?,
        funds: vec![],
    });

    staking.is_burned_by_admin = true;

    let _ = STAKINGS.save(deps.storage, staking.sender.clone(), &stakings_state);

    Ok(Response::new()
        .add_event(
            Event::new("Burned By Admin")
                .add_attribute("nft_addr", staking_info.nft_addr.clone())
                .add_attribute("token_id", staking_info.token_id.clone())
                .add_attribute("index", index.to_string()),
        )
        .add_message(burn_msg))
}

pub fn execute_claim(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    index: u64,
) -> Result<Response, ContractError> {
    let stakings_state = STAKINGS.may_load(deps.storage, info.sender.to_string().clone())?.unwrap();
    let staking_info = stakings_state[index as usize].clone();

    if info.sender != staking_info.sender {
        return Err(ContractError::Unauthorized {})
    }

    if staking_info.is_burned_by_admin {
        return Err(ContractError::StakedTokenIsBurnedByAdmin {})
    }

    if staking_info.end_timestamp > env.block.time {
        return Err(ContractError::StillUnbounding { will_finish: staking_info.end_timestamp })
    }

    let transfer_msg: CosmosMsg = CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: staking_info.nft_addr.clone(),
        msg: to_json_binary(&Cw721ExecuteMsg::TransferNft {
            recipient: info.sender.to_string().clone(),
            token_id: staking_info.token_id.clone(),
        })?,
        funds: vec![],
    });


    Ok(Response::new()
    .add_event(
        Event::new("Claimed")
            .add_attribute("nft_addr", staking_info.nft_addr.clone())
            .add_attribute("token_id", staking_info.token_id.clone())
            .add_attribute("index", index.to_string()),
    )
    .add_message(transfer_msg))
}

pub fn execute_add_collection(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo, 
    nft_addr: String,
) -> Result<Response, ContractError> {
    let mut config = CONFIG.load(deps.storage)?;

    let nft_addr = deps.api.addr_validate(&nft_addr)?;

    if info.sender != config.admin {
        return Err(ContractError::Unauthorized {});
    }

;
    if !config.whitelisted_nft_addrs.contains(&nft_addr) {
        config.whitelisted_nft_addrs.push(nft_addr.clone());
    } else {
        return Err(ContractError::AlreadyWhitelisted {});
    }

    let _ = CONFIG.save(deps.storage, &config);
    Ok(Response::new()
        .add_event(
            Event::new("Collection added")
                .add_attribute("nft_addr", nft_addr.clone())
    ))
}

pub fn execute_remove_collection(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo, 
    nft_addr: String,
) -> Result<Response, ContractError> {
    let mut config = CONFIG.load(deps.storage)?;

    if info.sender != config.admin {
        return Err(ContractError::Unauthorized {});
    }

    if let Some(index) = config.whitelisted_nft_addrs.iter().position(|nft_address| *nft_address == nft_addr) {
        config.whitelisted_nft_addrs.remove(index);
    } else {
        return Err(ContractError::NotWhitelisted {})
    }

    let _ = CONFIG.save(deps.storage, &config);

    Ok(Response::new()
    .add_event(
        Event::new("Collection removed")
            .add_attribute("nft_addr", nft_addr.clone())
    ))

}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::AdminAddress {} => to_json_binary(&query_admin(deps)?),
        QueryMsg::WhitelistedNftAddresses {} => to_json_binary(&query_whitelisted_nft_addresses(deps)?),
        QueryMsg::StakingsByAddress{ address } => to_json_binary(&query_stakings_by_address(deps, address)?),
    }
}

pub fn query_admin(deps: Deps) -> StdResult<AdminAddressResponse> {
    let config = CONFIG.load(deps.storage)?;
    Ok(AdminAddressResponse {
        admin: config.admin.to_string(),
    })
}

pub fn query_stakings_by_address(deps: Deps, address: String) -> StdResult<StakingsResponse> {
    let stakings: Vec<Staking>;
    let stakings_state = STAKINGS.may_load(deps.storage, address).unwrap();
    if stakings_state.is_some() {
        stakings = stakings_state.unwrap();
    } else {
        stakings = vec![];
    }
    Ok(StakingsResponse {
        stakings: stakings,
    })
}

pub fn query_whitelisted_nft_addresses(deps: Deps) -> StdResult<WhitelistedNftAddressesResponse> {
    let nft_addrs = CONFIG.load(deps.storage)?.whitelisted_nft_addrs;
    let nft_addrs_string: Vec<String> = nft_addrs.into_iter().map(|addr| addr.into_string()).collect();
    Ok(WhitelistedNftAddressesResponse {
        nft_addrs: nft_addrs_string,
    })

}