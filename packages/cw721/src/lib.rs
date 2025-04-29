pub mod error;
pub mod execute;
pub mod extension;
pub mod helpers;
#[allow(deprecated)]
pub mod msg;
pub mod query;
pub mod receiver;
pub mod state;
pub mod traits;

pub use cw_utils::Expiration;
use msg::{CollectionInfoAndExtensionResponse, NftExtensionMsg};
pub use state::{Approval, Attribute, CollectionExtension, NftExtension, RoyaltyInfo};

// Expose for 3rd party contracts interacting without a need to directly dependend on cw_ownable.
//
// `Action` is used in `Cw721ExecuteMsg::UpdateMinterOwnership` and `Cw721ExecuteMsg::UpdateCreatorOwnership`, `Ownership` is
// used in `Cw721QueryMsg::GetMinterOwnership`, `Cw721QueryMsg::GetCreatorOwnership`, and `OwnershipError` is used in
// `Cw721ContractError::Ownership`.
pub use cw_ownable::{Action, Ownership, OwnershipError};

// explicit type for better distinction.
#[deprecated(since = "0.19.0", note = "Please use `NftExtension` instead")]
pub type MetaData = NftExtension;
#[deprecated(
    since = "0.19.0",
    note = "Please use `CollectionInfoAndExtensionResponse<DefaultOptionalCollectionExtension>` instead"
)]
pub type ContractInfoResponse = CollectionInfoAndExtensionResponse;
#[cfg(test)]
pub mod testing;
