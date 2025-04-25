pub mod error;
pub mod msg;
pub mod state;

// expose so other libs dont need to import cw721
pub use cw721::*;

// These types are re-exported so that contracts interacting with this
// one don't need a direct dependency on cw_ownable to use the API.
//
// `Action` is used in `ExecuteMsg::UpdateMinterOwnership` and `ExecuteMsg::UpdateCreatorOwnership`, `Ownership` is
// used in `QueryMsg::GetMinterOwnership`, `QueryMsg::GetCreatorOwnership`, and `OwnershipError` is used in
// `ContractError::Ownership`.
pub use cw_ownable::{Action, Ownership, OwnershipError};
use extension::Cw721BaseExtensions;

// Version info for migration
pub const CONTRACT_NAME: &str = "crates.io:cw721-base";
pub const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[deprecated(
    since = "0.19.0",
    note = "Please use `EmptyOptionalNftExtension` instead"
)]
pub type Extension = EmptyOptionalNftExtension;

pub type Cw721BaseContract<'a> = Cw721BaseExtensions<'a>;

pub mod entry {

    use super::*;

    #[cfg(not(feature = "library"))]
    use cosmwasm_std::entry_point;
    use cosmwasm_std::{Binary, Deps, DepsMut, Env, MessageInfo, Response};
    use cw721::traits::{Cw721Execute, Cw721Query};
    use error::ContractError;
    use msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};

    #[cfg_attr(not(feature = "library"), entry_point)]
    pub fn instantiate(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        msg: InstantiateMsg,
    ) -> Result<Response, ContractError> {
        let contract = Cw721BaseContract::default();
        contract.instantiate_with_version(deps, &env, &info, msg, CONTRACT_NAME, CONTRACT_VERSION)
    }

    #[cfg_attr(not(feature = "library"), entry_point)]
    pub fn execute(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        msg: ExecuteMsg,
    ) -> Result<Response, ContractError> {
        let contract = Cw721BaseContract::default();
        contract.execute(deps, &env, &info, msg)
    }

    #[cfg_attr(not(feature = "library"), entry_point)]
    pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> Result<Binary, ContractError> {
        let contract = Cw721BaseContract::default();
        contract.query(deps, &env, msg)
    }

    #[cfg_attr(not(feature = "library"), entry_point)]
    pub fn migrate(deps: DepsMut, env: Env, msg: MigrateMsg) -> Result<Response, ContractError> {
        let contract = Cw721BaseContract::default();
        contract.migrate(deps, env, msg, CONTRACT_NAME, CONTRACT_VERSION)
    }
}