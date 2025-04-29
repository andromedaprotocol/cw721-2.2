use std::fmt::Debug;

use cosmwasm_std::{
    to_json_binary, Addr, Api, Binary, CosmosMsg, Deps, DepsMut, Empty, Env, MessageInfo,
    QuerierWrapper, Response, StdResult, Storage, WasmMsg, WasmQuery,
};
use cw_ownable::{Action, Ownership};
use cw_utils::Expiration;
use serde::{de::DeserializeOwned, Serialize};

#[allow(deprecated)]
use crate::{
    error::Cw721ContractError,
    execute::{
        approve, burn_nft, initialize_creator, initialize_minter, instantiate,
        instantiate_with_version, migrate, mint, revoke, send_nft, transfer_nft, update_nft_info,
    },
    msg::{
        AllNftInfoResponse, ApprovalResponse, ApprovalsResponse,
        CollectionInfoAndExtensionResponse, Cw721ExecuteMsg, Cw721InstantiateMsg, Cw721MigrateMsg,
        Cw721QueryMsg, MinterResponse, NftInfoResponse, NumTokensResponse, OperatorResponse,
        OperatorsResponse, OwnerOfResponse, TokensResponse,
    },
    query::{
        query_all_nft_info, query_all_tokens, query_approval, query_approvals,
        query_collection_extension_attributes, query_collection_info,
        query_collection_info_and_extension, query_creator_ownership, query_minter,
        query_minter_ownership, query_nft_info, query_num_tokens, query_operator, query_operators,
        query_owner_of, query_tokens, query_withdraw_address,
    },
    state::CollectionInfo,
    Attribute,
};
use crate::{
    execute::{
        approve_all, remove_withdraw_address, revoke_all, set_withdraw_address,
        update_creator_ownership, update_minter_ownership, withdraw_funds,
    },
    msg::{AllInfoResponse, ConfigResponse},
    query::{query_all_info, query_config},
    Approval,
};

/// This is an exact copy of `CustomMsg`, since implementing a trait for a type from another crate is not possible.
///
/// Possible:
/// `impl<T> Cw721CustomMsg for Option<T> where T: Cw721CustomMsg {}`
///
/// Not possible:
/// `impl<T> CustomMsg for Option<T> where T: CustomMsg {}`
///
/// This will be removed once the `CustomMsg` trait is moved to the `cosmwasm_std` crate: https://github.com/CosmWasm/cosmwasm/issues/2056

pub trait Cw721State: Serialize + DeserializeOwned + Clone + Debug {}

impl Cw721State for Empty {}
impl<T> Cw721State for Option<T> where T: Cw721State {}

/// e.g. for checking whether an NFT has specific traits (metadata).
pub trait Contains {
    fn contains(&self, other: &Self) -> bool;
}

pub trait StateFactory<TState> {
    fn create(
        &self,
        deps: Deps,
        env: &Env,
        info: Option<&MessageInfo>,
        current: Option<&TState>,
    ) -> Result<TState, Cw721ContractError>;
    fn validate(
        &self,
        deps: Deps,
        env: &Env,
        info: Option<&MessageInfo>,
        current: Option<&TState>,
    ) -> Result<(), Cw721ContractError>;
}

pub trait ToAttributesState {
    fn to_attributes_state(&self) -> Result<Vec<Attribute>, Cw721ContractError>;
}

impl<T> ToAttributesState for Option<T>
where
    T: ToAttributesState,
{
    fn to_attributes_state(&self) -> Result<Vec<Attribute>, Cw721ContractError> {
        match self {
            Some(inner) => inner.to_attributes_state(),
            None => Ok(vec![]),
        }
    }
}

pub trait FromAttributesState: Sized {
    fn from_attributes_state(value: &[Attribute]) -> Result<Self, Cw721ContractError>;
}

impl<T> FromAttributesState for Option<T>
where
    T: FromAttributesState,
{
    fn from_attributes_state(value: &[Attribute]) -> Result<Self, Cw721ContractError> {
        if value.is_empty() {
            Ok(None)
        } else {
            T::from_attributes_state(value).map(Some)
        }
    }
}

/// Trait with generic onchain nft and collection extensions used to execute the contract logic and contains default implementations for all messages.
pub trait Cw721Execute {
    fn instantiate_with_version(
        &self,
        deps: DepsMut,
        env: &Env,
        info: &MessageInfo,
        msg: Cw721InstantiateMsg,
        contract_name: &str,
        contract_version: &str,
    ) -> Result<Response, Cw721ContractError> {
        instantiate_with_version(deps, env, info, msg, contract_name, contract_version)
    }

    fn instantiate(
        &self,
        deps: DepsMut,
        env: &Env,
        info: &MessageInfo,
        msg: Cw721InstantiateMsg,
    ) -> Result<Response, Cw721ContractError> {
        instantiate(deps, env, info, msg)
    }

    fn execute(
        &self,
        deps: DepsMut,
        env: &Env,
        info: &MessageInfo,
        msg: Cw721ExecuteMsg,
    ) -> Result<Response, Cw721ContractError> {
        match msg {
            Cw721ExecuteMsg::Mint {
                token_id,
                owner,
                token_uri,
            } => self.mint(deps, env, info, token_id, owner, token_uri),
            Cw721ExecuteMsg::Approve {
                spender,
                token_id,
                expires,
            } => self.approve(deps, env, info, spender, token_id, expires),
            Cw721ExecuteMsg::Revoke { spender, token_id } => {
                self.revoke(deps, env, info, spender, token_id)
            }
            Cw721ExecuteMsg::ApproveAll { operator, expires } => {
                self.approve_all(deps, env, info, operator, expires)
            }
            Cw721ExecuteMsg::RevokeAll { operator } => self.revoke_all(deps, env, info, operator),
            Cw721ExecuteMsg::TransferNft {
                recipient,
                token_id,
            } => self.transfer_nft(deps, env, info, recipient, token_id),
            Cw721ExecuteMsg::SendNft {
                contract,
                token_id,
                msg,
            } => self.send_nft(deps, env, info, contract, token_id, msg),
            Cw721ExecuteMsg::Burn { token_id } => self.burn_nft(deps, env, info, token_id),
            #[allow(deprecated)]
            Cw721ExecuteMsg::UpdateOwnership(action) => {
                self.update_minter_ownership(deps, env, info, action)
            }
            Cw721ExecuteMsg::UpdateMinterOwnership(action) => {
                self.update_minter_ownership(deps, env, info, action)
            }
            Cw721ExecuteMsg::UpdateCreatorOwnership(action) => {
                self.update_creator_ownership(deps, env, info, action)
            }
            Cw721ExecuteMsg::SetWithdrawAddress { address } => {
                self.set_withdraw_address(deps, &info.sender, address)
            }
            Cw721ExecuteMsg::RemoveWithdrawAddress {} => {
                self.remove_withdraw_address(deps.storage, &info.sender)
            }
            Cw721ExecuteMsg::WithdrawFunds { amount } => self.withdraw_funds(deps.storage, &amount),
        }
    }

    fn migrate(
        &self,
        deps: DepsMut,
        env: Env,
        msg: Cw721MigrateMsg,
        contract_name: &str,
        contract_version: &str,
    ) -> Result<Response, Cw721ContractError> {
        migrate(deps, env, msg, contract_name, contract_version)
    }

    // ------- ERC721-based functions -------
    fn transfer_nft(
        &self,
        deps: DepsMut,
        env: &Env,
        info: &MessageInfo,
        recipient: String,
        token_id: String,
    ) -> Result<Response, Cw721ContractError> {
        transfer_nft(deps, env, info, &recipient, &token_id)?;

        Ok(Response::new()
            .add_attribute("action", "transfer_nft")
            .add_attribute("sender", info.sender.to_string())
            .add_attribute("recipient", recipient)
            .add_attribute("token_id", token_id))
    }

    fn send_nft(
        &self,
        deps: DepsMut,
        env: &Env,
        info: &MessageInfo,
        contract: String,
        token_id: String,
        msg: Binary,
    ) -> Result<Response, Cw721ContractError> {
        send_nft(deps, env, info, contract, token_id, msg)
    }

    fn approve(
        &self,
        deps: DepsMut,
        env: &Env,
        info: &MessageInfo,
        spender: String,
        token_id: String,
        expires: Option<Expiration>,
    ) -> Result<Response, Cw721ContractError> {
        approve(deps, env, info, spender, token_id, expires)
    }

    fn revoke(
        &self,
        deps: DepsMut,
        env: &Env,
        info: &MessageInfo,
        spender: String,
        token_id: String,
    ) -> Result<Response, Cw721ContractError> {
        revoke(deps, env, info, spender, token_id)
    }

    fn approve_all(
        &self,
        deps: DepsMut,
        env: &Env,
        info: &MessageInfo,
        operator: String,
        expires: Option<Expiration>,
    ) -> Result<Response, Cw721ContractError> {
        approve_all(deps, env, info, operator, expires)
    }

    fn revoke_all(
        &self,
        deps: DepsMut,
        _env: &Env,
        info: &MessageInfo,
        operator: String,
    ) -> Result<Response, Cw721ContractError> {
        revoke_all(deps, _env, info, operator)
    }

    fn burn_nft(
        &self,
        deps: DepsMut,
        env: &Env,
        info: &MessageInfo,
        token_id: String,
    ) -> Result<Response, Cw721ContractError> {
        burn_nft(deps, env, info, token_id)
    }

    // ------- opionated cw721 functions -------
    fn initialize_creator(
        &self,
        storage: &mut dyn Storage,
        api: &dyn Api,
        creator: Option<&str>,
    ) -> StdResult<Ownership<Addr>> {
        initialize_creator(storage, api, creator)
    }

    fn initialize_minter(
        &self,
        storage: &mut dyn Storage,
        api: &dyn Api,
        minter: Option<&str>,
    ) -> StdResult<Ownership<Addr>> {
        initialize_minter(storage, api, minter)
    }

    #[allow(clippy::too_many_arguments)]
    fn mint(
        &self,
        deps: DepsMut,
        env: &Env,
        info: &MessageInfo,
        token_id: String,
        owner: String,
        token_uri: Option<String>,
    ) -> Result<Response, Cw721ContractError> {
        mint(deps, env, info, token_id, owner, token_uri)
    }

    fn update_minter_ownership(
        &self,
        deps: DepsMut,
        env: &Env,
        info: &MessageInfo,
        action: Action,
    ) -> Result<Response, Cw721ContractError> {
        update_minter_ownership(deps, env, info, action)
    }

    fn update_creator_ownership(
        &self,
        deps: DepsMut,
        env: &Env,
        info: &MessageInfo,
        action: Action,
    ) -> Result<Response, Cw721ContractError> {
        update_creator_ownership(deps, env, info, action)
    }

    /// The creator is the only one eligible to update NFT's token uri and onchain metadata (`NftInfo.extension`).
    /// NOTE: approvals and owner are not affected by this call, since they belong to the NFT owner.
    fn update_nft_info(
        &self,
        deps: DepsMut,
        env: &Env,
        info: &MessageInfo,
        token_id: String,
        token_uri: Option<String>,
    ) -> Result<Response, Cw721ContractError> {
        update_nft_info(deps, env, info.into(), token_id, token_uri)
    }

    fn set_withdraw_address(
        &self,
        deps: DepsMut,
        sender: &Addr,
        address: String,
    ) -> Result<Response, Cw721ContractError> {
        set_withdraw_address(deps, sender, address)
    }

    fn remove_withdraw_address(
        &self,
        storage: &mut dyn Storage,
        sender: &Addr,
    ) -> Result<Response, Cw721ContractError> {
        remove_withdraw_address(storage, sender)
    }

    fn withdraw_funds(
        &self,
        storage: &mut dyn Storage,
        amount: &cosmwasm_std::Coin,
    ) -> Result<Response, Cw721ContractError> {
        withdraw_funds(storage, amount)
    }
}

/// Trait with generic onchain nft and collection extensions used to query the contract state and contains default implementations for all queries.
pub trait Cw721Query {
    fn query(
        &self,
        deps: Deps,
        env: &Env,
        msg: Cw721QueryMsg,
    ) -> Result<Binary, Cw721ContractError> {
        match msg {
            #[allow(deprecated)]
            Cw721QueryMsg::Minter {} => Ok(to_json_binary(&self.query_minter(deps.storage)?)?),
            #[allow(deprecated)]
            Cw721QueryMsg::ContractInfo {} => Ok(to_json_binary(
                &self.query_collection_info_and_extension(deps)?,
            )?),
            Cw721QueryMsg::GetConfig {} => Ok(to_json_binary(
                &self.query_all_collection_info(deps, env.contract.address.to_string())?,
            )?),
            Cw721QueryMsg::GetCollectionInfoAndExtension {} => Ok(to_json_binary(
                &self.query_collection_info_and_extension(deps)?,
            )?),
            Cw721QueryMsg::GetAllInfo {} => Ok(to_json_binary(&self.query_all_info(deps, env)?)?),
            Cw721QueryMsg::GetCollectionExtensionAttributes {} => Ok(to_json_binary(
                &self.query_collection_extension_attributes(deps)?,
            )?),
            Cw721QueryMsg::NftInfo { token_id } => Ok(to_json_binary(
                &self.query_nft_info(deps.storage, token_id)?,
            )?),
            Cw721QueryMsg::OwnerOf {
                token_id,
                include_expired,
            } => Ok(to_json_binary(&self.query_owner_of(
                deps,
                env,
                token_id,
                include_expired.unwrap_or(false),
            )?)?),
            Cw721QueryMsg::AllNftInfo {
                token_id,
                include_expired,
            } => Ok(to_json_binary(&self.query_all_nft_info(
                deps,
                env,
                token_id,
                include_expired.unwrap_or(false),
            )?)?),
            Cw721QueryMsg::Operator {
                owner,
                operator,
                include_expired,
            } => Ok(to_json_binary(&self.query_operator(
                deps,
                env,
                owner,
                operator,
                include_expired.unwrap_or(false),
            )?)?),
            Cw721QueryMsg::AllOperators {
                owner,
                include_expired,
                start_after,
                limit,
            } => Ok(to_json_binary(&self.query_operators(
                deps,
                env,
                owner,
                include_expired.unwrap_or(false),
                start_after,
                limit,
            )?)?),
            Cw721QueryMsg::NumTokens {} => {
                Ok(to_json_binary(&self.query_num_tokens(deps.storage)?)?)
            }
            Cw721QueryMsg::Tokens {
                owner,
                start_after,
                limit,
            } => Ok(to_json_binary(&self.query_tokens(
                deps,
                env,
                owner,
                start_after,
                limit,
            )?)?),
            Cw721QueryMsg::AllTokens { start_after, limit } => Ok(to_json_binary(
                &self.query_all_tokens(deps, env, start_after, limit)?,
            )?),
            Cw721QueryMsg::Approval {
                token_id,
                spender,
                include_expired,
            } => Ok(to_json_binary(&self.query_approval(
                deps,
                env,
                token_id,
                spender,
                include_expired.unwrap_or(false),
            )?)?),
            Cw721QueryMsg::Approvals {
                token_id,
                include_expired,
            } => Ok(to_json_binary(&self.query_approvals(
                deps,
                env,
                token_id,
                include_expired.unwrap_or(false),
            )?)?),
            #[allow(deprecated)]
            Cw721QueryMsg::Ownership {} => {
                Ok(to_json_binary(&self.query_minter_ownership(deps.storage)?)?)
            }
            Cw721QueryMsg::GetMinterOwnership {} => {
                Ok(to_json_binary(&self.query_minter_ownership(deps.storage)?)?)
            }
            Cw721QueryMsg::GetCreatorOwnership {} => Ok(to_json_binary(
                &self.query_creator_ownership(deps.storage)?,
            )?),
            Cw721QueryMsg::GetWithdrawAddress {} => {
                Ok(to_json_binary(&self.query_withdraw_address(deps)?)?)
            }
        }
    }

    #[deprecated(since = "0.19.0", note = "Please use query_minter_ownership instead")]
    /// Deprecated: use query_minter_ownership instead! Will be removed in next release!
    fn query_minter(&self, storage: &dyn Storage) -> StdResult<MinterResponse> {
        #[allow(deprecated)]
        query_minter(storage)
    }

    fn query_minter_ownership(&self, storage: &dyn Storage) -> StdResult<Ownership<Addr>> {
        query_minter_ownership(storage)
    }

    fn query_creator_ownership(&self, storage: &dyn Storage) -> StdResult<Ownership<Addr>> {
        query_creator_ownership(storage)
    }

    fn query_collection_info(&self, deps: Deps) -> StdResult<CollectionInfo> {
        query_collection_info(deps.storage)
    }

    fn query_collection_extension_attributes(&self, deps: Deps) -> StdResult<Vec<Attribute>> {
        query_collection_extension_attributes(deps)
    }

    fn query_all_collection_info(
        &self,
        deps: Deps,
        contract_addr: impl Into<String>,
    ) -> Result<ConfigResponse, Cw721ContractError> {
        query_config(deps, contract_addr)
    }

    fn query_collection_info_and_extension(
        &self,
        deps: Deps,
    ) -> Result<CollectionInfoAndExtensionResponse, Cw721ContractError> {
        query_collection_info_and_extension(deps)
    }

    fn query_all_info(&self, deps: Deps, env: &Env) -> StdResult<AllInfoResponse> {
        query_all_info(deps, env)
    }

    fn query_num_tokens(&self, storage: &dyn Storage) -> StdResult<NumTokensResponse> {
        query_num_tokens(storage)
    }

    fn query_nft_info(
        &self,
        storage: &dyn Storage,
        token_id: String,
    ) -> StdResult<NftInfoResponse> {
        query_nft_info(storage, token_id)
    }

    fn query_owner_of(
        &self,
        deps: Deps,
        env: &Env,
        token_id: String,
        include_expired_approval: bool,
    ) -> StdResult<OwnerOfResponse> {
        query_owner_of(deps, env, token_id, include_expired_approval)
    }

    /// operator returns the approval status of an operator for a given owner if exists
    fn query_operator(
        &self,
        deps: Deps,
        env: &Env,
        owner: String,
        operator: String,
        include_expired_approval: bool,
    ) -> StdResult<OperatorResponse> {
        query_operator(deps, env, owner, operator, include_expired_approval)
    }

    /// operators returns all operators owner given access to
    fn query_operators(
        &self,
        deps: Deps,
        env: &Env,
        owner: String,
        include_expired_approval: bool,
        start_after: Option<String>,
        limit: Option<u32>,
    ) -> StdResult<OperatorsResponse> {
        query_operators(
            deps,
            env,
            owner,
            include_expired_approval,
            start_after,
            limit,
        )
    }

    fn query_approval(
        &self,
        deps: Deps,
        env: &Env,
        token_id: String,
        spender: String,
        include_expired_approval: bool,
    ) -> StdResult<ApprovalResponse> {
        query_approval(
            deps,
            env,
            token_id,
            deps.api.addr_validate(&spender)?,
            include_expired_approval,
        )
    }

    /// approvals returns all approvals owner given access to
    fn query_approvals(
        &self,
        deps: Deps,
        env: &Env,
        token_id: String,
        include_expired_approval: bool,
    ) -> StdResult<ApprovalsResponse> {
        query_approvals(deps, env, token_id, include_expired_approval)
    }

    fn query_tokens(
        &self,
        deps: Deps,
        _env: &Env,
        owner: String,
        start_after: Option<String>,
        limit: Option<u32>,
    ) -> StdResult<TokensResponse> {
        query_tokens(deps, _env, owner, start_after, limit)
    }

    fn query_all_tokens(
        &self,
        deps: Deps,
        _env: &Env,
        start_after: Option<String>,
        limit: Option<u32>,
    ) -> StdResult<TokensResponse> {
        query_all_tokens(deps, _env, start_after, limit)
    }

    fn query_all_nft_info(
        &self,
        deps: Deps,
        env: &Env,
        token_id: String,
        include_expired_approval: bool,
    ) -> StdResult<AllNftInfoResponse> {
        query_all_nft_info(deps, env, token_id, include_expired_approval)
    }

    fn query_withdraw_address(&self, deps: Deps) -> StdResult<Option<String>> {
        query_withdraw_address(deps)
    }
}

/// Generic trait with onchain nft and collection extensions used to call query and execute messages for a given CW721 addr.
pub trait Cw721Calls<TNftExtension, TCollectionExtension>
where
    TNftExtension: Cw721State,
    TCollectionExtension: Cw721State,
{
    /// Returns the CW721 address.
    fn addr(&self) -> Addr;

    /// Executes the CW721 contract with the given message.
    fn call(&self, msg: Cw721ExecuteMsg) -> StdResult<CosmosMsg> {
        let msg = to_json_binary(&msg)?;
        Ok(WasmMsg::Execute {
            contract_addr: self.addr().into(),
            msg,
            funds: vec![],
        }
        .into())
    }

    /// Queries the CW721 contract with the given message.
    fn query<T: DeserializeOwned>(
        &self,
        querier: &QuerierWrapper,
        req: Cw721QueryMsg,
    ) -> StdResult<T> {
        let query = WasmQuery::Smart {
            contract_addr: self.addr().into(),
            msg: to_json_binary(&req)?,
        }
        .into();
        querier.query(&query)
    }

    /*** queries ***/
    fn owner_of<T: Into<String>>(
        &self,
        querier: &QuerierWrapper,
        token_id: T,
        include_expired: bool,
    ) -> StdResult<OwnerOfResponse> {
        let req = Cw721QueryMsg::OwnerOf {
            token_id: token_id.into(),
            include_expired: Some(include_expired),
        };
        self.query(querier, req)
    }

    fn approval<T: Into<String>>(
        &self,
        querier: &QuerierWrapper,
        token_id: T,
        spender: T,
        include_expired: Option<bool>,
    ) -> StdResult<ApprovalResponse> {
        let req = Cw721QueryMsg::Approval {
            token_id: token_id.into(),
            spender: spender.into(),
            include_expired,
        };
        let res: ApprovalResponse = self.query(querier, req)?;
        Ok(res)
    }

    fn approvals<T: Into<String>>(
        &self,
        querier: &QuerierWrapper,
        token_id: T,
        include_expired: Option<bool>,
    ) -> StdResult<ApprovalsResponse> {
        let req = Cw721QueryMsg::Approvals {
            token_id: token_id.into(),
            include_expired,
        };
        let res: ApprovalsResponse = self.query(querier, req)?;
        Ok(res)
    }

    fn all_operators<T: Into<String>>(
        &self,
        querier: &QuerierWrapper,
        owner: T,
        include_expired: bool,
        start_after: Option<String>,
        limit: Option<u32>,
    ) -> StdResult<Vec<Approval>> {
        let req = Cw721QueryMsg::AllOperators {
            owner: owner.into(),
            include_expired: Some(include_expired),
            start_after,
            limit,
        };
        let res: OperatorsResponse = self.query(querier, req)?;
        Ok(res.operators)
    }

    fn num_tokens(&self, querier: &QuerierWrapper) -> StdResult<u64> {
        let req = Cw721QueryMsg::NumTokens {};
        let res: NumTokensResponse = self.query(querier, req)?;
        Ok(res.count)
    }

    /// This is a helper to get the metadata and extension data in one call
    fn config<U: DeserializeOwned>(&self, querier: &QuerierWrapper) -> StdResult<ConfigResponse> {
        let req = Cw721QueryMsg::GetConfig {};
        self.query(querier, req)
    }

    /// This is a helper to get the metadata and extension data in one call
    fn collection_info<U: DeserializeOwned>(
        &self,
        querier: &QuerierWrapper,
    ) -> StdResult<CollectionInfoAndExtensionResponse> {
        let req = Cw721QueryMsg::GetCollectionInfoAndExtension {};
        self.query(querier, req)
    }

    /// With NFT onchain metadata
    fn nft_info<T: Into<String>, U: DeserializeOwned>(
        &self,
        querier: &QuerierWrapper,
        token_id: T,
    ) -> StdResult<NftInfoResponse> {
        let req = Cw721QueryMsg::NftInfo {
            token_id: token_id.into(),
        };
        self.query(querier, req)
    }

    /// With NFT onchain metadata
    fn all_nft_info<T: Into<String>, U: DeserializeOwned>(
        &self,
        querier: &QuerierWrapper,
        token_id: T,
        include_expired: bool,
    ) -> StdResult<AllNftInfoResponse> {
        let req = Cw721QueryMsg::AllNftInfo {
            token_id: token_id.into(),
            include_expired: Some(include_expired),
        };
        self.query(querier, req)
    }

    /// With enumerable extension
    fn tokens<T: Into<String>>(
        &self,
        querier: &QuerierWrapper,
        owner: T,
        start_after: Option<String>,
        limit: Option<u32>,
    ) -> StdResult<TokensResponse> {
        let req = Cw721QueryMsg::Tokens {
            owner: owner.into(),
            start_after,
            limit,
        };
        self.query(querier, req)
    }

    /// With enumerable extension
    fn all_tokens(
        &self,
        querier: &QuerierWrapper,
        start_after: Option<String>,
        limit: Option<u32>,
    ) -> StdResult<TokensResponse> {
        let req = Cw721QueryMsg::AllTokens { start_after, limit };
        self.query(querier, req)
    }

    /// returns true if the contract supports the metadata extension
    fn has_metadata(&self, querier: &QuerierWrapper) -> bool {
        self.collection_info::<Empty>(querier).is_ok()
    }

    /// returns true if the contract supports the enumerable extension
    fn has_enumerable(&self, querier: &QuerierWrapper) -> bool {
        self.tokens(querier, self.addr(), None, Some(1)).is_ok()
    }
}
