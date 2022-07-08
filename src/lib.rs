use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::UnorderedMap;
use near_sdk::{
    env, near_bindgen, AccountId, BorshStorageKey, PanicOnDefault, Promise, PromiseOrValue,
};

mod storage_impl;

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
    claimed_rewards: UnorderedMap<AccountId, u128>,
    owner_id: AccountId,
}

#[derive(BorshSerialize, BorshStorageKey)]
enum StorageKeys {
    ClaimedRewards,
}

#[near_bindgen]
impl Contract {
    #[init]
    pub fn new() -> Self {
        Self {
            claimed_rewards: UnorderedMap::new(StorageKeys::ClaimedRewards),
            owner_id: env::predecessor_account_id(),
        }
    }
}
