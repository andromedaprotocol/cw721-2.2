use crate::state::Cw721Config;

/// Opionated version of generic `Cw721Extensions` with `EmptyOptionalNftExtension` and `DefaultOptionalCollectionExtension` using:
/// - `Empty` for NftInfo extension (onchain metadata).
/// - `Empty` for NftInfo extension msg for onchain metadata.
/// - `DefaultOptionalCollectionExtension` for CollectionInfo extension (onchain attributes).
/// - `DefaultOptionalCollectionExtensionMsg` for CollectionInfo extension msg for onchain collection attributes.
/// - `Empty` for custom extension msg for custom contract logic.
/// - `Empty` for custom query msg for custom contract logic.
/// - `Empty` for custom response msg for custom contract logic.
pub struct Cw721BaseExtensions<'a> {
    pub config: Cw721Config<'a>,
}

impl Default for Cw721BaseExtensions<'static> {
    fn default() -> Self {
        Self {
            config: Cw721Config::default(),
        }
    }
}
