use schemars::JsonSchema;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::marker::PhantomData;

use cosmwasm_std::{Addr, BlockInfo, CustomMsg, StdResult, Storage};

use cw721::{ContractInfoResponse, Expiration};
use cw_storage_plus::{Index, IndexList, IndexedMap, Item, Map, MultiIndex};

pub struct Cw721Contract<'a, MintExt, ResponseExt, InstantiateExt, ExecuteExt, QueryExt>
where
    MintExt: Serialize + DeserializeOwned + Clone,
    InstantiateExt: CustomMsg + DeserializeOwned,
    QueryExt: CustomMsg,
    ExecuteExt: CustomMsg,
{
    pub contract_info: Item<'a, ContractInfoResponse<InstantiateExt>>,
    pub minter: Item<'a, Addr>,
    pub token_count: Item<'a, u64>,
    /// Stored as (granter, operator) giving operator full control over granter's account
    pub operators: Map<'a, (&'a Addr, &'a Addr), Expiration>,
    pub tokens: IndexedMap<'a, &'a str, TokenInfo<MintExt>, TokenIndexes<'a, MintExt>>,

    pub(crate) _custom_response: PhantomData<ResponseExt>,
    pub(crate) _custom_instantiate: PhantomData<InstantiateExt>,
    pub(crate) _custom_query: PhantomData<QueryExt>,
    pub(crate) _custom_execute: PhantomData<ExecuteExt>,
}

impl<MintExt, ResponseExt, InstantiateExt, ExecuteExt, QueryExt> Default
    for Cw721Contract<'static, MintExt, ResponseExt, InstantiateExt, ExecuteExt, QueryExt>
where
    MintExt: Serialize + DeserializeOwned + Clone,
    InstantiateExt: CustomMsg + DeserializeOwned,
    ExecuteExt: CustomMsg,
    QueryExt: CustomMsg,
{
    fn default() -> Self {
        Self::new(
            "nft_info",
            "minter",
            "num_tokens",
            "operators",
            "tokens",
            "tokens__owner",
        )
    }
}

impl<'a, MintExt, ResponseExt, InstantiateExt, ExecuteExt, QueryExt>
    Cw721Contract<'a, MintExt, ResponseExt, InstantiateExt, ExecuteExt, QueryExt>
where
    MintExt: Serialize + DeserializeOwned + Clone,
    InstantiateExt: CustomMsg + DeserializeOwned,
    ExecuteExt: CustomMsg,
    QueryExt: CustomMsg,
{
    fn new(
        contract_key: &'a str,
        minter_key: &'a str,
        token_count_key: &'a str,
        operator_key: &'a str,
        tokens_key: &'a str,
        tokens_owner_key: &'a str,
    ) -> Self {
        let indexes = TokenIndexes {
            owner: MultiIndex::new(token_owner_idx, tokens_key, tokens_owner_key),
        };
        Self {
            contract_info: Item::new(contract_key),
            minter: Item::new(minter_key),
            token_count: Item::new(token_count_key),
            operators: Map::new(operator_key),
            tokens: IndexedMap::new(tokens_key, indexes),
            _custom_response: PhantomData,
            _custom_execute: PhantomData,
            _custom_query: PhantomData,
            _custom_instantiate: PhantomData,
        }
    }

    pub fn token_count(&self, storage: &dyn Storage) -> StdResult<u64> {
        Ok(self.token_count.may_load(storage)?.unwrap_or_default())
    }

    pub fn increment_tokens(&self, storage: &mut dyn Storage) -> StdResult<u64> {
        let val = self.token_count(storage)? + 1;
        self.token_count.save(storage, &val)?;
        Ok(val)
    }

    pub fn decrement_tokens(&self, storage: &mut dyn Storage) -> StdResult<u64> {
        let val = self.token_count(storage)? - 1;
        self.token_count.save(storage, &val)?;
        Ok(val)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct TokenInfo<MintExt> {
    /// The owner of the newly minted NFT
    pub owner: Addr,
    /// Approvals are stored here, as we clear them all upon transfer and cannot accumulate much
    pub approvals: Vec<Approval>,

    /// Universal resource identifier for this NFT
    /// Should point to a JSON file that conforms to the ERC721
    /// Metadata JSON Schema
    pub token_uri: Option<String>,

    /// You can add any custom metadata here when you extend cw721-base
    pub extension: MintExt,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct Approval {
    /// Account that can transfer/send the token
    pub spender: Addr,
    /// When the Approval expires (maybe Expiration::never)
    pub expires: Expiration,
}

impl Approval {
    pub fn is_expired(&self, block: &BlockInfo) -> bool {
        self.expires.is_expired(block)
    }
}

pub struct TokenIndexes<'a, MintExt>
where
    MintExt: Serialize + DeserializeOwned + Clone,
{
    pub owner: MultiIndex<'a, Addr, TokenInfo<MintExt>, String>,
}

impl<'a, MintExt> IndexList<TokenInfo<MintExt>> for TokenIndexes<'a, MintExt>
where
    MintExt: Serialize + DeserializeOwned + Clone,
{
    fn get_indexes(&'_ self) -> Box<dyn Iterator<Item = &'_ dyn Index<TokenInfo<MintExt>>> + '_> {
        let v: Vec<&dyn Index<TokenInfo<MintExt>>> = vec![&self.owner];
        Box::new(v.into_iter())
    }
}

pub fn token_owner_idx<MintExt>(_pk: &[u8], d: &TokenInfo<MintExt>) -> Addr {
    d.owner.clone()
}
