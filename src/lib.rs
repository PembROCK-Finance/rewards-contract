use near_contract_standards::fungible_token::core::ext_ft_core;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::{
    assert_one_yocto, collections::UnorderedMap, env, json_types::U128, near_bindgen, require,
    AccountId, Balance, BorshStorageKey, Gas, PanicOnDefault, Promise, PromiseError,
    PromiseOrValue, StorageUsage,
};
use pembrock_integration::{ext_pembrock, AccountInfo};

mod pembrock_integration;
mod storage_impl;

const GAS_FOR_FT_TRANSFER_CALLBACK: Gas = Gas(10_000_000_000_000);
const GAS_FOR_FT_TRANSFER: Gas = Gas(10_000_000_000_000);
const GAS_FOR_GET_ACCOUNT_CALLBACK: Gas =
    Gas(5_000_000_000_000 + GAS_FOR_FT_TRANSFER.0 + GAS_FOR_FT_TRANSFER_CALLBACK.0);
const GAS_FOR_GET_ACCOUNT: Gas = Gas(25_000_000_000_000);
const GAS_FOR_CLAIM: Gas =
    Gas(25_000_000_000_000 + GAS_FOR_GET_ACCOUNT.0 + GAS_FOR_GET_ACCOUNT_CALLBACK.0);

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
    /// AccountID of contract owner.
    owner_id: AccountId,

    /// AccountID of Pembrock contract.
    pembrock_id: AccountId,

    /// AccountID of PEM token contract.
    pem_token_id: AccountId,

    /// AccountID -> Claimed rewards.
    claimed_rewards: UnorderedMap<AccountId, Balance>,

    /// The storage size in bytes for one account.
    account_storage_usage: StorageUsage,
}

#[derive(BorshSerialize, BorshStorageKey)]
enum StorageKey {
    ClaimedRewards,
}

#[near_bindgen]
impl Contract {
    #[init]
    pub fn new(pembrock_id: AccountId, pem_token_id: AccountId) -> Self {
        let mut this = Self {
            owner_id: env::predecessor_account_id(),
            pembrock_id,
            pem_token_id,
            claimed_rewards: UnorderedMap::new(StorageKey::ClaimedRewards),
            account_storage_usage: 0,
        };
        this.measure_account_storage_usage();
        this
    }

    fn measure_account_storage_usage(&mut self) {
        let initial_storage_usage = env::storage_usage();
        let tmp_account_id = AccountId::new_unchecked("a".repeat(64));
        self.claimed_rewards.insert(&tmp_account_id, &0u128);
        self.account_storage_usage = env::storage_usage() - initial_storage_usage;
        self.claimed_rewards.remove(&tmp_account_id);
    }

    fn is_account_registered(&self, account_id: &AccountId) -> bool {
        self.claimed_rewards.keys().any(|aid| aid == *account_id)
    }

    fn register_account(&mut self, account_id: &AccountId) {
        self.claimed_rewards.insert(account_id, &0);
    }

    ///
    pub fn owner_withdraw(&mut self, amount: U128) -> Promise {
        assert_one_yocto();

        require!(
            env::predecessor_account_id() == self.owner_id,
            "Not an owner"
        );

        ext_ft_core::ext(self.pem_token_id.clone())
            .with_static_gas(GAS_FOR_FT_TRANSFER)
            .with_attached_deposit(1)
            .ft_transfer(self.owner_id.clone(), amount, None)
    }

    ///
    pub fn get_claimed_rewards(&self, account_id: &AccountId) -> U128 {
        self.claimed_rewards
            .get(&account_id)
            .unwrap_or_default()
            .into()
    }

    ///
    /// Returns claimed amount
    pub fn claim(&self) -> Promise {
        assert_one_yocto();

        require!(env::prepaid_gas() >= GAS_FOR_CLAIM, "More gas is required");

        let account_id = env::predecessor_account_id();
        require!(
            self.is_account_registered(&account_id),
            "Account is not registered"
        );

        ext_pembrock::ext(self.pembrock_id.clone())
            .with_static_gas(GAS_FOR_GET_ACCOUNT)
            .get_account(&account_id)
            .then(
                Self::ext(env::current_account_id())
                    .with_static_gas(GAS_FOR_GET_ACCOUNT_CALLBACK)
                    .get_account_callback(account_id),
            )
            .into()
    }

    #[private]
    pub fn get_account_callback(
        &mut self,
        account_id: AccountId,
        #[callback_result] account_info: Result<AccountInfo, PromiseError>,
    ) -> PromiseOrValue<U128> {
        let total_rewards: u128 = match account_info {
            Ok(info) => info.total_rewards.into(),
            _ => 0,
        };

        let claimed_rewards = self.claimed_rewards.get(&account_id).unwrap_or_default();
        if total_rewards <= claimed_rewards {
            return PromiseOrValue::Value(0.into());
        }

        let unclaimed_rewards = total_rewards - claimed_rewards;
        self.claimed_rewards.insert(&account_id, &total_rewards);

        ext_ft_core::ext(self.pem_token_id.clone())
            .with_static_gas(GAS_FOR_FT_TRANSFER)
            .with_attached_deposit(1)
            .ft_transfer(account_id.clone(), unclaimed_rewards.into(), None)
            .then(
                Self::ext(env::current_account_id())
                    .with_static_gas(GAS_FOR_FT_TRANSFER_CALLBACK)
                    .ft_transfer_callback(
                        account_id,
                        claimed_rewards.into(),
                        unclaimed_rewards.into(),
                    ),
            )
            .into()
    }

    #[private]
    pub fn ft_transfer_callback(
        &mut self,
        account_id: AccountId,
        claimed_rewards: U128,
        unclaimed_rewards: U128,
        #[callback_result] result: Result<(), PromiseError>,
    ) -> U128 {
        if result.is_ok() {
            return unclaimed_rewards;
        }

        self.claimed_rewards
            .insert(&account_id, &claimed_rewards.into());

        0.into()
    }
}
