use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    env::predecessor_account_id,
    json_types::U128,
    log, require,
    store::UnorderedMap,
    AccountId, BorshStorageKey,
};

#[derive(BorshSerialize, BorshDeserialize, BorshStorageKey)]
pub enum StorageKey {
    Balance,
    Allowed,
}

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct ERC20 {
    pub name: String,
    pub symbol: String,
    pub decimals: u8,
    pub total_supply: u128,
    pub balance: UnorderedMap<AccountId, u128>,
    pub allowed: UnorderedMap<AccountId, UnorderedMap<AccountId, u128>>,
}

impl ERC20 {
    pub fn init(name: String, symbol: String, decimals: u8, total_supply: U128) -> Self {
        Self {
            name,
            symbol,
            decimals,
            total_supply: total_supply.into(),
            balance: UnorderedMap::new(StorageKey::Balance),
            allowed: UnorderedMap::new(StorageKey::Allowed),
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn symbol(&self) -> &str {
        &self.symbol
    }

    pub fn decimals(&self) -> &u8 {
        &self.decimals
    }

    pub fn total_supply(&self) -> &u128 {
        &self.total_supply
    }

    pub fn balance_of(&self, account_id: AccountId) -> Option<&u128> {
        self.balance.get(&account_id)
    }

    pub fn transfer(&mut self, to: AccountId, value: U128) -> bool {
        let user_balance = self.balance_of(predecessor_account_id()).unwrap_or(&0u128);
        let value = value.into();
        require!(*user_balance >= value);
        self.balance
            .insert(predecessor_account_id(), user_balance - value);

        let mut receiver_balance = self.balance_of(to.clone()).unwrap_or(&0u128);
        if let 0 = receiver_balance {
            self.balance.insert(predecessor_account_id().clone(), 0u128);
            receiver_balance = &0u128;
        }

        self.balance.insert(to, receiver_balance + value);

        true
    }

    pub fn transfer_from(&mut self, from: AccountId, to: AccountId, value: U128) -> bool {
        let user_balance = self.balance_of(from.clone()).unwrap();
        let value = value.into();
        require!(*user_balance >= value);
        require!(self.allowance(from.clone(), predecessor_account_id()) >= &value);
        self.balance.insert(from, user_balance - value).unwrap();

        let mut receiver_balance = self.balance_of(to.clone()).unwrap_or(&0u128);
        if let 0 = receiver_balance {
            self.balance.insert(predecessor_account_id().clone(), 0u128);
            receiver_balance = &0u128;
        }

        self.balance.insert(to, receiver_balance + value).unwrap();

        true
    }

    pub fn approve(&mut self, spender: AccountId, value: U128) {
        let allowance_exist = self.allowed.contains_key(&predecessor_account_id());
        if let false = allowance_exist {
            self.allowed.insert(
                predecessor_account_id(),
                UnorderedMap::new(near_sdk::env::keccak256(spender.as_bytes())),
            );
        }

        self.allowed
            .get_mut(&predecessor_account_id())
            .unwrap()
            .insert(spender, value.into());
    }

    pub fn allowance(&self, owner: AccountId, spender: AccountId) -> &u128 {
        self.allowed.get(&owner).unwrap().get(&spender).unwrap()
    }

    pub fn mint(&mut self, to: AccountId, value: U128) {
        log!("Mint!");
        if let false = self.balance.contains_key(&to) {
            self.balance.insert(to.clone(), 0);
        }
        *self.balance.get_mut(&to).unwrap() += value.0;
    }

    pub fn burn(&mut self, account_id: AccountId, value: U128) {
        require!(value.0 != 0);
        require!(*self.balance_of(account_id.clone()).unwrap_or(&0u128) >= value.0);
        *self.balance.get_mut(&account_id).unwrap() -= value.0;
    }
}

#[cfg(test)]
mod tests {
    use crate::*;
    use near_sdk::{base64::encode, test_utils::VMContextBuilder, testing_env};

    const DECIMALS: u8 = 18;
    const TOTAL_SUPPLY: u128 = 10u128.pow(9);

    fn get_context(predecessor: String) -> VMContextBuilder {
        let mut builder = VMContextBuilder::new();
        builder.predecessor_account_id(predecessor.parse().unwrap());
        builder
    }

    #[test]
    fn test_approve() {
        let predecessor = "nutinaguti.testnet".parse().unwrap();
        let context = get_context(predecessor);
        testing_env!(context.build());

        let mut contract = ERC20::init(
            "FUN COIN".to_string(),
            "FUNC".to_string(),
            DECIMALS,
            TOTAL_SUPPLY.into(),
        );
        contract.approve("test.testnet".parse().unwrap(), 1.into());
        let allowance = contract.allowance(
            "nutinaguti.testnet".parse().unwrap(),
            "test.testnet".parse().unwrap(),
        );
        assert_eq!(1, *allowance);

        contract.approve("test.testnet".parse().unwrap(), 2.into());
        let allowance = contract.allowance(
            "nutinaguti.testnet".parse().unwrap(),
            "test.testnet".parse().unwrap(),
        );
        assert_eq!(2, *allowance);
    }

    #[test]
    #[should_panic]
    fn test_transfer_negative() {
        let predecessor = "nutinaguti.testnet".parse().unwrap();
        let context = get_context(predecessor);
        testing_env!(context.build());

        let mut contract = ERC20::init(
            "FUN COIN".to_string(),
            "FUNC".to_string(),
            DECIMALS,
            TOTAL_SUPPLY.into(),
        );
        contract.transfer("test.testnet".parse().unwrap(), 1.into());
    }

    #[test]
    fn test_transfer_positive() {
        let predecessor = "nutinaguti.testnet".parse().unwrap();
        let context = get_context(predecessor);
        testing_env!(context.build());

        let mut contract = ERC20::init(
            "FUN COIN".to_string(),
            "FUNC".to_string(),
            DECIMALS,
            TOTAL_SUPPLY.into(),
        );
        contract.mint("nutinaguti.testnet".parse().unwrap(), 1.into());
        contract.transfer("test.testnet".parse().unwrap(), 1.into());
        assert_eq!(
            0u128,
            *contract
                .balance_of("nutinaguti.testnet".parse().unwrap())
                .unwrap()
        );
        assert_eq!(
            1u128,
            *contract
                .balance_of("test.testnet".parse().unwrap())
                .unwrap()
        );
    }

    #[test]
    #[should_panic]
    fn test_transfer_from_negative() {
        let predecessor = "nutinaguti.testnet".parse().unwrap();
        let context = get_context(predecessor);
        testing_env!(context.build());

        let mut contract = ERC20::init(
            "FUN COIN".to_string(),
            "FUNC".to_string(),
            DECIMALS,
            TOTAL_SUPPLY.into(),
        );
        contract.mint("test.testnet".parse().unwrap(), 1.into());
        contract.transfer_from(
            "test.testnet".parse().unwrap(),
            "nutinaguti.testnet".parse().unwrap(),
            1.into(),
        );
    }

    #[test]
    fn test_transfer_from_positive() {
        let predecessor = "nutinaguti.testnet".parse().unwrap();
        let context = get_context(predecessor);

        let predecessor_2 = "test.testnet".parse().unwrap();
        let context_2 = get_context(predecessor_2);
        testing_env!(context.build());

        let mut contract = ERC20::init(
            "FUN COIN".to_string(),
            "FUNC".to_string(),
            DECIMALS,
            TOTAL_SUPPLY.into(),
        );
        contract.mint("test.testnet".parse().unwrap(), 1.into());

        testing_env!(context_2.build());
        contract.approve("nutinaguti.testnet".parse().unwrap(), 1.into());
        testing_env!(context.build());

        contract.transfer_from(
            "test.testnet".parse().unwrap(),
            "nutinaguti.testnet".parse().unwrap(),
            1.into(),
        );
    }
}
