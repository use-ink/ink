//! ## Automated Money Market (AMM)
//!
//! An AMM is a decentralized marketplace to swap tokens. Usually swap fees are
//! significantly cheaper than centralized exchanges, and this system allows anyone to
//! participate as either the Swapper (a user interested in swapping one token for
//! another) or as a Liquidity Provide (LP), //! who is financially incentivised to
//! deposit equal amounts of token1 and token2. Each time a token is swapped a small
//! percentage is given to all the LPs, and each share is proportional to the amount of
//! liquidity that a LP provides. Pretty simple, right? You can learn more about AMMs
//! [here](https://www.coindesk.com/learn/2021/08/20/what-is-an-automated-market-maker/).
//!
//! This code example was inspired by
//! [this](https://learn.figment.io/tutorials/build-polkadot-amm-using-ink) blog.
//!
//! ## Warning
//!
//! This contract is an *example*. It is neither audited nor endorsed for production use.
//! Do **not** rely on it to keep any
//!
//! ## Tokens and AMMs
//!
//! If you are familiar with Solidity you may have worked with AMM contracts that
//! integrate multiple ERC20 contracts for token pairs. This is a composable contract
//! model. In this code example, however, we demonstrate a more simple version where all
//! tokens are maintained by this contract (one contract instead of many composed
//! contracts).
//!
//! ## The Faucet
//!
//! There is a `faucet()` function built in to this contract to fund your wallet balance.
//! You can use this to //! test dive the features in a staging environment, and in tests.
//!
//! ## Error Handling
//!
//! Any function that modifies the state returns a `Result` type and does not changes the
//! state if the `Error` occurs. The errors are defined as an `enum` type. Any other error
//! or invariant violation triggers a panic and therefore rolls back the transaction.
//!

#![cfg_attr(not(feature = "std"), no_std)]

use ink_lang as ink;

#[ink::contract]
mod amm {
    use ink_storage::{traits::SpreadAllocate, Mapping};

    /// Errors are returned for the following conditions:
    /// ZeroLiquidity - the caller does not have any LP tokens
    /// ZeroAmount - the caller has a zero balance for the token in question
    /// InsufficientAmount - the caller sends a transaction exceeding their balance
    /// NonEquivalentAmount - an LP deposits two tokens with unequal value
    /// ThresholdNotReached - the calculated LP token amount for a contribution is 0
    /// InvalidLPAmount - a caller withdraws more than the value of their LP balance
    /// InsufficientLiquidity - swap amount exceeds what is available in the pool
    /// SlippageExceeded - the max or min slippage exceeds the threshold set by the caller
    #[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
        ZeroLiquidity,
        ZeroAmount,
        InsufficientAmount,
        NonEquivalentAmount,
        ThresholdNotReached,
        InvalidLPAmount,
        InsufficientLiquidity,
        SlippageExceeded,
    }

    /// We create an ergonomic helper so we can do things like Result<Balance> instead of
    /// Result<Balance, Error>
    pub type Result<T> = core::result::Result<T, Error>;

    /// Amm
    /// This contract maintains all three tokens: token1, token2, and lp_tokens
    /// total_lp_tokens - the aggregate total of all LP tokens created / burned
    /// total_token1 - the total liquidity for token1
    /// total_token2 - the total liquidity for token2
    /// total1_balances - a ledger containing all user balances for token1
    /// total2_balances - a ledger containing all user balances for token2
    /// lp_token_balances - a ledger containing all user balances for the LP token
    /// fee - the LP cut for all swaps made in the pool
    #[derive(Default, SpreadAllocate)]
    #[ink(storage)]
    pub struct Amm {
        total_lp_tokens: LPTokens,
        total_token1: Token1,
        total_token2: Token2,
        token1_balances: Mapping<AccountId, Token1>,
        token2_balances: Mapping<AccountId, Token2>,
        lp_token_balances: Mapping<AccountId, LPTokens>,
        fee: Balance,
        precision: u128,
    }

    /// Here we create type aliases for readibility
    type Token1 = Balance;
    type Token2 = Balance;
    type LPTokens = Balance;

    /// Struct to hold Account balances
    #[derive(
        Default, Copy, PartialEq, Eq, Debug, Clone, scale::Decode, scale::Encode,
    )]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub struct Balances {
        token1: Token1,
        token2: Token2,
        lp_tokens: LPTokens,
    }

    /// Struct to summarize the total token counts and swap fee
    #[derive(
        Default, Copy, PartialEq, Eq, Debug, Clone, scale::Decode, scale::Encode,
    )]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub struct PoolDetails {
        total_token1: Token1,
        total_token2: Token2,
        total_lp_tokens: LPTokens,
        fee: Balance,
    }

    /// Used to describe how much of each token was minted
    #[ink(event)]
    pub struct TokensMinted {
        token1_amount: Token1,
        token2_amount: Token2,
    }

    /// Used to describe if token1 was swapped for Token2 or if Token2 was swapped for
    /// Token1
    #[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum SwapType {
        Token1ForToken2,
        Token2ForToken1,
    }

    /// Used to describe how much of each token was deposited
    /// amount_token1 - how many of token1 was deposited by the liquidity provider
    /// amount_token2 - how many of token2 was deposited by the liquidity provider
    /// amount_lp - how many lp_tokens were given in exchange for both tokens
    #[ink(event)]
    pub struct LiquidityProvided {
        token1_amount: Token1,
        token2_amount: Token2,
        amount_lp: LPTokens,
    }

    /// Used to describe how much of each token was withdrawn from the pool
    /// amount_token1 - how many of token1 was withdrawn by the liquidity provider
    /// amount_token2 - how many of token2 was withdrawn by the liquidity provider
    /// amount_lp - how many lp_tokens were burned in exchange for both tokens
    #[ink(event)]
    pub struct LiquidityWithdrawn {
        token1_amount: Token1,
        token2_amount: Token2,
        amount_lp: LPTokens,
    }

    /// Used to describe how many tokens were swapped
    /// deposit_amount - the amount of Token1 or Token2 that was given.
    /// swap_amount - the amount of Token1 or Token2 that was received. If Token1 was deposited
    /// swap_type - Used to describe which token was deposited
    #[ink(event)]
    pub struct TokenSwapped {
        deposit_amount: Token1,
        swap_amount: Token2,
        swap_type: SwapType,
    }

    #[ink(impl)]
    impl Amm {
        // ==============================================================================
        // 1. Constructor
        // ==============================================================================
        /// The constructor takes two arguments: fee, and precision. fee it used to
        /// calculate the swap fee to be given to liquidity providers (LPs). Precision is used
        /// as a multiplier for more granular fractions of shares for LPs.
        ///
        /// NOTE:
        /// We init this struct using a function. Simply calling
        /// ink_lang::utils::initialize_contract(1000) will not create the correct storage
        /// keys "under the hood" for tests. The function format is required to initialize
        /// struct members the right way.
        #[ink(constructor)]
        pub fn new(fee: Balance, precision: u128) -> Self {
            ink_lang::utils::initialize_contract(|contract: &mut Self| {
                contract.fee = if fee >= 1000 { 0 } else { fee };
                contract.precision = if fee < 1_000_000 {
                    1_000_000
                } else {
                    precision
                }
            })
        }

        // ==============================================================================
        // 2. Public Interactions
        // ==============================================================================

        /// faucet() is only used in a test scenario. Use this to add balances to your
        /// wallet for token1 and token2
        #[ink(message)]
        pub fn faucet(&mut self, token1_amount: Token1, token2_amount: Token2) {
            let caller = self.env().caller();
            let token1 = self.token1_balances.get(&caller).unwrap_or(0);
            let token2 = self.token2_balances.get(&caller).unwrap_or(0);

            self.token1_balances
                .insert(&caller, &(token1 + token1_amount));
            self.token2_balances
                .insert(&caller, &(token2 + token2_amount));

            self.env().emit_event(TokensMinted {
                token1_amount,
                token2_amount,
            })
        }

        /// To become a LP and start earning on swap rewards you must put liquidity into
        /// the pool using this function. NOTE: The first time you call this you must make
        /// sure that the two token amounts are of equal value. The first deposit is
        /// crucial to get right because it sets up a mathematical relationship between
        /// the two tokens assuming that they are of equal value.
        #[ink(message)]
        pub fn provide_liquidity(
            &mut self,
            token1_amount: Token1,
            token2_amount: Token2,
        ) -> Result<LPTokens> {
            self.has_balance(&self.token1_balances, token1_amount)?;
            self.has_balance(&self.token2_balances, token2_amount)?;

            // If the total lp token amount is 0 it means that this is the first time
            // someone is providing liquidity, and the total lp tokens (shares) will be
            // initialized at a constant amount.
            let lp_tokens_for_deposit = if self.total_lp_tokens == 0 {
                100 * self.precision
            } else {
                let first_lp_amount =
                    self.total_lp_tokens * token1_amount / self.total_token1;
                let second_lp_amount =
                    self.total_lp_tokens * token2_amount / self.total_token2;

                if first_lp_amount != second_lp_amount {
                    return Err(Error::NonEquivalentAmount);
                }
                first_lp_amount
            };

            if lp_tokens_for_deposit == 0 {
                return Err(Error::ThresholdNotReached);
            }

            let caller = self.env().caller();
            let token1 = self.token1_balances.get(&caller).unwrap_or(0);
            let token2 = self.token2_balances.get(&caller).unwrap_or(0);
            self.token1_balances
                .insert(&caller, &(token1 - token1_amount));
            self.token2_balances
                .insert(&caller, &(token2 - token2_amount));

            self.total_token1 += token1_amount;
            self.total_token2 += token2_amount;
            self.total_lp_tokens += lp_tokens_for_deposit;

            let lp_token_balance = self.lp_token_balances.get(&caller).unwrap_or(0);
            self.lp_token_balances
                .insert(&caller, &(lp_token_balance + lp_tokens_for_deposit));

            self.env().emit_event(LiquidityProvided {
                token1_amount,
                token2_amount,
                amount_lp: lp_tokens_for_deposit,
            });

            Ok(lp_tokens_for_deposit)
        }

        /// This is used for LP to redeem the liquidity plus interest earned in exchange
        /// for LP tokens
        #[ink(message)]
        pub fn withdraw(&mut self, claim: LPTokens) -> Result<(Token1, Token2)> {
            let caller = self.env().caller();
            self.has_balance(&self.lp_token_balances, claim)?;

            let (token1_amount, token2_amount) = self.get_withdraw_estimate(claim)?;

            self.total_lp_tokens -= claim;
            self.total_token1 -= token1_amount;
            self.total_token2 -= token2_amount;

            let lp_token_balance = self.lp_token_balances.get(&caller).unwrap_or(0);
            self.lp_token_balances
                .insert(&caller, &(lp_token_balance - claim));

            let t1_balance = self.token1_balances.get(&caller).unwrap_or(0);
            self.token1_balances
                .insert(&caller, &(token1_amount + t1_balance));

            let t2_balance = self.token2_balances.get(&caller).unwrap_or(0);
            self.token2_balances
                .insert(&caller, &(token2_amount + t2_balance));

            self.env().emit_event(LiquidityWithdrawn {
                token1_amount,
                token2_amount,
                amount_lp: claim,
            });

            Ok((token1_amount, token2_amount))
        }

        /// Swap token1 in exchange for token2. You can pass in the min_token2_required to
        /// guard yourself from losing too much due to "slippage".
        #[ink(message)]
        pub fn swap_from_token1_to_token2(
            &mut self,
            token1_amount: Balance,
            min_token2_required: Balance,
        ) -> Result<Balance> {
            let caller = self.env().caller();
            self.has_balance(&self.token1_balances, token1_amount)?;

            let token2_amount = self.get_token1_estimate_given_token1(token1_amount)?;
            if token2_amount < min_token2_required {
                return Err(Error::SlippageExceeded);
            }
            let t1_balance = self.token1_balances.get(&caller).unwrap_or(0);
            self.token1_balances
                .insert(&caller, &(t1_balance - token1_amount));

            self.total_token1 += token1_amount;
            self.total_token2 -= token2_amount;

            let t2_balance = self.token2_balances.get(&caller).unwrap_or(0);
            self.token2_balances
                .insert(&caller, &(t2_balance + token2_amount));

            self.env().emit_event(TokenSwapped {
                deposit_amount: token1_amount,
                swap_amount: token2_amount,
                swap_type: SwapType::Token1ForToken2,
            });

            Ok(token2_amount)
        }

        /// Swap token2 in exchange for token1. You can pass in the max_token1_required to
        /// guard yourself from losing too much due to "slippage".
        #[ink(message)]
        pub fn swap_from_token2_to_token1(
            &mut self,
            token2_amount: Balance,
            max_token1_required: Balance,
        ) -> Result<Balance> {
            let caller = self.env().caller();
            let token1_amount =
                self.get_swap_token1_estimate_given_token2(token2_amount)?;
            if token1_amount > max_token1_required {
                return Err(Error::SlippageExceeded);
            }
            self.has_balance(&self.token1_balances, token1_amount)?;

            let token1_balance = self.token1_balances.get(&caller).unwrap_or(0);
            self.token1_balances
                .insert(&caller, &(token1_balance - token1_amount));

            self.total_token1 += token1_amount;
            self.total_token2 -= token2_amount;

            let token2_balance = self.token2_balances.get(&caller).unwrap_or(0);
            self.token2_balances
                .insert(&caller, &(token2_balance + token2_amount));

            self.env().emit_event(TokenSwapped {
                deposit_amount: token2_amount,
                swap_amount: token1_amount,
                swap_type: SwapType::Token2ForToken1,
            });

            Ok(token1_amount)
        }

        // ==============================================================================
        // 3. Public Read API
        // ==============================================================================

        /// Returns the balances of the caller
        #[ink(message)]
        pub fn balances(&self) -> Balances {
            let caller = self.env().caller();
            Balances {
                token1: self.token1_balances.get(&caller).unwrap_or(0),
                token2: self.token2_balances.get(&caller).unwrap_or(0),
                lp_tokens: self.lp_token_balances.get(&caller).unwrap_or(0),
            }
        }

        /// Returns the total pool amount for token1, token2, and lp_tokens, and the pool
        /// swap fee
        #[ink(message)]
        pub fn pool_details(&self) -> PoolDetails {
            PoolDetails {
                total_token1: self.total_token1,
                total_token2: self.total_token2,
                total_lp_tokens: self.total_lp_tokens,
                fee: self.fee,
            }
        }

        /// Returns the amount of token1 a caller can receive in exchange for the given
        /// amount of token2. This does not account for the swap fee. Returns an error if
        /// the pool does not have any liquidity.
        #[ink(message)]
        pub fn get_equivalent_token1_estimate(
            &self,
            amount_token2: Balance,
        ) -> Result<Balance> {
            self.has_active_pool()?;
            Ok(self.total_token1 * amount_token2 / self.total_token2)
        }

        /// Returns the amount of token2 a caller can receive in exchange for the given
        /// amount of token1. This does not account for the swap fee. Returns an error if
        /// the pool does not have any liquidity.
        #[ink(message)]
        pub fn get_equivalent_token2_estimate(
            &self,
            amount_token1: Balance,
        ) -> Result<Balance> {
            self.has_active_pool()?;
            Ok(self.total_token2 * amount_token1 / self.total_token1)
        }

        /// Returns the amount of token1 and token2 a LP will receive in exchange for LP
        /// tokens. This result should increase over time based on swap activity assuming
        /// that the pool fee is greater than 0
        #[ink(message)]
        pub fn get_withdraw_estimate(
            &self,
            lp_tokens: LPTokens,
        ) -> Result<(Token1, Token2)> {
            self.has_active_pool()?;
            if lp_tokens > self.total_lp_tokens {
                return Err(Error::InvalidLPAmount);
            }

            let amount_token1 = lp_tokens * self.total_token1 / self.total_lp_tokens;
            let amount_token2 = lp_tokens * self.total_token2 / self.total_lp_tokens;
            Ok((amount_token1, amount_token2))
        }

        /// Get the exchange amount for token2 given the amount in token1. This accounts
        /// for the pool fee. Returns an error if the pool liquidity is zero
        #[ink(message)]
        pub fn get_token1_estimate_given_token1(
            &self,
            amount_token1: Balance,
        ) -> Result<Balance> {
            self.has_active_pool()?;

            let amount_token1 = (1000 - self.fee) * amount_token1 / 1000;
            let new_token1_total = self.total_token1 + amount_token1;
            let new_token2_total = self.constant_k() / new_token1_total;
            let mut amount_token2 = self.total_token2 - new_token2_total;

            // To ensure that Token2's pool is not completely depleted leading to inf:0
            // ratio
            if amount_token2 == self.total_token2 {
                amount_token2 -= 1;
            }

            Ok(amount_token2)
        }

        /// Get the exchange amount for token1 given the amount in token2. This accounts
        /// for the pool fee. Returns an error if the pool liquidity is zero
        #[ink(message)]
        pub fn get_swap_token1_estimate_given_token2(
            &self,
            token2_amount: Balance,
        ) -> Result<Balance> {
            self.has_active_pool()?;
            if token2_amount >= self.total_token2 {
                return Err(Error::InsufficientLiquidity);
            }

            let token2_after = self.total_token2 - token2_amount;
            let token1_after = self.constant_k() / token2_after;
            let amount_token1 =
                (token1_after - self.total_token1) * 1000 / (1000 - self.fee);
            Ok(amount_token1)
        }

        // ==============================================================================
        // 4. Private Helpers
        // ==============================================================================

        fn has_balance(
            &self,
            token: &Mapping<AccountId, Balance>,
            amount: Balance,
        ) -> Result<()> {
            let caller = self.env().caller();
            let balance = token.get(&caller).unwrap_or(0);

            match amount {
                0 => Err(Error::ZeroAmount),
                _ if amount > balance => Err(Error::InsufficientAmount),
                _ => Ok(()),
            }
        }

        fn constant_k(&self) -> Balance {
            self.total_token1 * self.total_token2
        }

        fn has_active_pool(&self) -> Result<()> {
            match self.constant_k() {
                0 => Err(Error::ZeroLiquidity),
                _ => Ok(()),
            }
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use ink_lang as ink;

        #[ink::test]
        fn can_create_amm() {
            let contract = Amm::new(0, 1_000_000);

            assert_eq!(
                contract.balances(),
                Balances {
                    token1: 0,
                    token2: 0,
                    lp_tokens: 0,
                }
            );
            assert_eq!(contract.pool_details(), PoolDetails::default());
        }

        #[ink::test]
        fn faucet_increases_token_counts() {
            let mut contract = Amm::new(0, 1_000_000);
            contract.faucet(100, 200);

            assert_eq!(
                contract.balances(),
                Balances {
                    token1: 100,
                    token2: 200,
                    lp_tokens: 0
                }
            );
        }

        #[ink::test]
        fn token_calculation_returns_error_with_zero_liquidity() {
            let contract = Amm::new(0, 1_000_000);
            let res = contract.get_equivalent_token1_estimate(5);

            assert_eq!(res, Err(Error::ZeroLiquidity));
        }

        #[ink::test]
        fn user_can_provide_liquidity() {
            let precision = 1_000_000;
            let mut contract = Amm::new(0, precision);
            contract.faucet(100, 200);
            let lp_tokens = contract.provide_liquidity(10, 20).unwrap();
            let expected_initial_lp_balance = 100 * precision;

            assert_eq!(lp_tokens, 100_000_000);
            assert_eq!(
                contract.pool_details(),
                PoolDetails {
                    total_token1: 10,
                    total_token2: 20,
                    total_lp_tokens: expected_initial_lp_balance,
                    fee: 0
                }
            );
            assert_eq!(
                contract.balances(),
                Balances {
                    token1: 90,
                    token2: 180,
                    lp_tokens: expected_initial_lp_balance,
                }
            );
        }

        #[ink::test]
        fn lp_can_withdraw_funds() {
            let mut contract = Amm::new(0, 1_000_000);
            contract.faucet(100, 200);
            contract.provide_liquidity(10, 20).unwrap();
            let portion_of_lp_tokens = 20_000_000;
            let remaining_lp_tokens = 80_000_000;

            assert_eq!(contract.withdraw(portion_of_lp_tokens).unwrap(), (2, 4));
            assert_eq!(
                contract.balances(),
                Balances {
                    token1: 92,
                    token2: 184,
                    lp_tokens: remaining_lp_tokens,
                }
            );
            assert_eq!(
                contract.pool_details(),
                PoolDetails {
                    total_token1: 8,
                    total_token2: 16,
                    total_lp_tokens: remaining_lp_tokens,
                    fee: 0
                }
            );
        }

        #[ink::test]
        fn users_can_swap_tokens() {
            let mut contract = Amm::new(0, 1_000_000);
            contract.faucet(100, 200);
            let lp_tokens = contract.provide_liquidity(50, 100).unwrap();

            let amount_token2 = contract.swap_from_token1_to_token2(50, 50).unwrap();

            assert_eq!(amount_token2, 50);
            assert_eq!(
                contract.balances(),
                Balances {
                    token1: 0,
                    token2: 150,
                    lp_tokens
                }
            );
            assert_eq!(
                contract.pool_details(),
                PoolDetails {
                    total_token1: 100,
                    total_token2: 50,
                    total_lp_tokens: lp_tokens,
                    fee: 0
                }
            );
        }

        #[ink::test]
        fn exceeding_max_slippage_returns_error() {
            let mut contract = Amm::new(0, 1_000_000);
            contract.faucet(100, 200);
            let lp_tokens = contract.provide_liquidity(50, 100).unwrap();

            let amount_token2 = contract.swap_from_token1_to_token2(50, 51);

            assert_eq!(amount_token2, Err(Error::SlippageExceeded));
            assert_eq!(
                contract.balances(),
                Balances {
                    token1: 50,
                    token2: 100,
                    lp_tokens
                }
            );
            assert_eq!(
                contract.pool_details(),
                PoolDetails {
                    total_token1: 50,
                    total_token2: 100,
                    total_lp_tokens: lp_tokens,
                    fee: 0
                }
            );
        }

        #[ink::test]
        fn swap_price_calculcations_are_correct() {
            let mut contract = Amm::new(100, 1_000_000);
            contract.faucet(100, 200);
            contract.provide_liquidity(50, 100).unwrap();

            let amount_token2 = contract.get_token1_estimate_given_token1(50).unwrap();

            assert_eq!(amount_token2, 48);
        }
    }
}
