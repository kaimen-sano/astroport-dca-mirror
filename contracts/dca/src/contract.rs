use std::str::FromStr;

use crate::error::ContractError;
use crate::handlers::{
    add_bot_tip, cancel_dca_order, create_dca_order, perform_dca_purchase, update_config,
    update_user_config, withdraw,
};
use crate::queries::{get_config, get_user_config, get_user_dca_orders};
use crate::state::{Config, CONFIG};

use astroport::asset::addr_validate_to_lower;
use cosmwasm_std::{
    entry_point, to_binary, Binary, Decimal, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
};

use astroport::dca::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};
use cw2::set_contract_version;

/// Contract name that is used for migration.
const CONTRACT_NAME: &str = "astroport-dca";
/// Contract version that is used for migration.
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

/// ## Description
/// Creates a new contract with the specified parameters in [`InstantiateMsg`].
/// Returns a [`Response`] with the specified attributes if the operation was successful,
/// or a [`ContractError`] if the contract was not created.
/// ## Params
/// * **deps** is an object of type [`DepsMut`].
///
/// * **env** is an object of type [`Env`].
///
/// * **_info** is an object of type [`MessageInfo`].
/// * **msg** is a message of type [`InstantiateMsg`] which contains the parameters for creating the contract.

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    // get max spread in decimal form
    let max_spread = Decimal::from_str(&msg.max_spread)?;

    // validate that factory_addr and router_addr is an address
    let factory_addr = addr_validate_to_lower(deps.api, &msg.factory_addr)?;
    let router_addr = addr_validate_to_lower(deps.api, &msg.router_addr)?;

    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let config = Config {
        max_hops: msg.max_hops,
        per_hop_fee: msg.per_hop_fee,
        whitelisted_tokens: msg.whitelisted_tokens,
        max_spread,
        factory_addr,
        router_addr,
    };

    CONFIG.save(deps.storage, &config)?;

    Ok(Response::new())
}

/// ## Description
/// Used for contract migration. Returns a default object of type [`Response`].
/// ## Params
/// * **_deps** is an object of type [`DepsMut`].
///
/// * **_env** is an object of type [`Env`].
///
/// * **_msg** is an object of type [`MigrateMsg`].
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(_deps: DepsMut, _env: Env, _msg: MigrateMsg) -> StdResult<Response> {
    Ok(Response::default())
}

/// ## Description
/// Exposes all the execute functions available in the contract.
/// ## Params
/// * **deps** is an object of type [`Deps`].
///
/// * **env** is an object of type [`Env`].
///
/// * **info** is an object of type [`MessageInfo`].
///
/// * **msg** is an object of type [`ExecuteMsg`].
///
/// ## Execution Messages
/// * **ExecuteMsg::UpdateConfig {
///         max_hops,
///         per_hop_fee,
///         whitelisted_tokens,
///         max_spread
///     }** Updates the contract configuration with the specified input parameters.
///
/// * **ExecuteMsg::Receive(msg)** Receives a message of type [`Cw20ReceiveMsg`] and processes
/// it depending on the received template.
///
/// * **ExecuteMsg::UpdateUserConfig {
///             assets,
///             slippage_tolerance,
///             auto_stake,
///             receiver,
///     }** Partially updates a users configuration with the new input parameters.
///
/// * **ExecuteMsg::AddBotTip { }** Adds some tip to fund DCA purchases.
///
/// * **ExecuteMsg::Withdraw { tip }** Withdraws a bot tip from the contract.
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::UpdateConfig {
            max_hops,
            per_hop_fee,
            whitelisted_tokens,
            max_spread,
        } => update_config(
            deps,
            info,
            max_hops,
            per_hop_fee,
            whitelisted_tokens,
            max_spread,
        ),
        ExecuteMsg::UpdateUserConfig {
            max_hops,
            max_spread,
        } => update_user_config(deps, info, max_hops, max_spread),
        ExecuteMsg::CreateDcaOrder {
            initial_asset,
            target_asset,
            interval,
            dca_amount,
        } => create_dca_order(
            deps,
            env,
            info,
            initial_asset,
            target_asset,
            interval,
            dca_amount,
        ),
        ExecuteMsg::AddBotTip {} => add_bot_tip(deps, info),
        ExecuteMsg::Withdraw { tip: amount } => withdraw(deps, info, amount),
        ExecuteMsg::PerformDcaPurchase { user, hops } => {
            perform_dca_purchase(deps, env, info, user, hops)
        }
        ExecuteMsg::CancelDcaOrder { initial_asset } => cancel_dca_order(deps, info, initial_asset),
    }
}

/// ## Description
/// Exposes all the queries available in the contract.
/// ## Params
/// * **deps** is an object of type [`Deps`].
///
/// * **_env** is an object of type [`Env`].
///
/// * **msg** is an object of type [`QueryMsg`].
///
/// ## Queries
/// * **QueryMsg::Config {}** Returns information about the configuration of the contract in a
/// [`Config`] object.
///
/// * **QueryMsg::UserConfig {}** Returns information about a specified users configuration set for
/// DCA purchases in a [`UserConfig`] object.
///
/// * **QueryMsg::UserDcaOrders {}** Returns information about a specified users current DCA orders
/// set in a [`Vec<UserDcaOrderResponse>`] object.
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&get_config(deps)?),
        QueryMsg::UserConfig { user } => to_binary(&get_user_config(deps, user)?),
        QueryMsg::UserDcaOrders { user } => to_binary(&get_user_dca_orders(deps, env, user)?),
    }
}