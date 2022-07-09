use near_sdk::{ext_contract, json_types::U128, serde::Deserialize, AccountId};

#[derive(Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct AccountInfo {
    pub total_rewards: U128,
}

#[ext_contract(ext_pembrock)]
pub trait PembrockContract {
    ///
    fn get_account(&self, account_id: &AccountId) -> AccountInfo;
}
