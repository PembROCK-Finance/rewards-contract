use near_contract_standards::storage_management::{
    StorageBalance, StorageBalanceBounds, StorageManagement,
};
use near_sdk::{env, json_types::U128, log, near_bindgen, AccountId, Balance};

use crate::*;

#[near_bindgen]
impl StorageManagement for Contract {
    #[payable]
    #[allow(unused_variables)]
    fn storage_deposit(
        &mut self,
        account_id: Option<AccountId>,
        registration_only: Option<bool>,
    ) -> StorageBalance {
        let amount: Balance = env::attached_deposit();
        let min_balance = self.storage_balance_bounds().min.0;
        let account_id = account_id.unwrap_or_else(env::predecessor_account_id);
        let refund = if self.is_account_registered(&account_id) {
            log!("The account is already registered, refunding the deposit");
            amount
        } else {
            require!(
                amount >= min_balance,
                "The attached deposit is less than the minimum storage balance"
            );
            self.register_account(&account_id);
            amount - min_balance
        };
        if refund > 0 {
            Promise::new(env::predecessor_account_id()).transfer(refund);
        }
        StorageBalance {
            total: min_balance.into(),
            available: 0.into(),
        }
    }

    #[payable]
    #[allow(unused_variables)]
    fn storage_withdraw(&mut self, amount: Option<U128>) -> StorageBalance {
        unimplemented!()
    }

    #[payable]
    #[allow(unused_variables)]
    fn storage_unregister(&mut self, force: Option<bool>) -> bool {
        unimplemented!()
    }

    fn storage_balance_bounds(&self) -> StorageBalanceBounds {
        let required_storage_balance =
            Balance::from(self.account_storage_usage) * env::storage_byte_cost();
        StorageBalanceBounds {
            min: required_storage_balance.into(),
            max: Some(required_storage_balance.into()),
        }
    }

    fn storage_balance_of(&self, account_id: AccountId) -> Option<StorageBalance> {
        if self.is_account_registered(&account_id) {
            Some(StorageBalance {
                total: self.storage_balance_bounds().min,
                available: 0.into(),
            })
        } else {
            None
        }
    }
}
