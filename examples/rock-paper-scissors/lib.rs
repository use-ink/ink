#![cfg_attr(not(feature = "std"), no_std)]

use ink_lang as ink;

mod game;

#[macro_export]
macro_rules! require {
    ($cond:expr, $err:expr) => {
        if !$cond {
            return Err($err)
        }
    };
}

#[ink::contract]
mod rock_paper_scissors {

    use ink_lang::codegen::initialize_contract;
    use ink_storage::{
        traits::SpreadAllocate,
        Mapping,
    };
    use scale::{
        Decode,
        Encode,
    };

    use crate::game::{
        Cancel,
        Create,
        Fail,
        Game,
        GameState,
        Hand,
        Join,
        Outcome,
        Proof,
        Transition,
        Update,
    };

    #[derive(Encode, Decode, Debug, PartialEq, Eq)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
        /// The game update message can not be processed because some preconditions were not met.
        Move(Fail),
        /// The bet size entered was too small (it can not be zero).
        BetSizeTooSmall,
        /// There is currently no active game for the submitted game ID.
        InvalidGameId,
        /// Random number generation triggered a collision. Try your message again.
        TryAgain,
        /// The caller is not allowed to execute this message.
        Permission,
        /// An error happened during balance transfer.
        Transfer,
    }

    pub type Result<T> = core::result::Result<T, Error>;

    /// Emitted when the state of a game changes.
    #[ink(event)]
    pub struct GameEvent {
        #[ink(topic)]
        caller: AccountId,
        #[ink(topic)]
        game_id: Hash,
        game: Game,
    }

    #[ink(storage)]
    #[derive(Default, SpreadAllocate)]
    pub struct RockPaperScissors {
        owner: AccountId,
        min_bet_size: Balance,
        games: Mapping<Hash, Game>,
    }

    impl RockPaperScissors {
        /// Constructor for the contract.
        ///
        /// `min_bet_size` is the minimum amount of balance needed for a player to create or join a game.
        #[ink(constructor)]
        pub fn new(min_bet_size: Balance) -> Self {
            assert!(min_bet_size > 0);
            initialize_contract(|c: &mut Self| {
                c.owner = c.env().caller();
                c.min_bet_size = min_bet_size;
                c.games = <Mapping<Hash, Game>>::default();
            })
        }

        /// Read the minimum bet size.
        #[ink(message)]
        pub fn get_min_bet_size(&self) -> Balance {
            self.min_bet_size
        }

        /// Set the minimum bet size.
        #[ink(message)]
        pub fn set_min_bet_size(&mut self, size: Balance) -> Result<()> {
            require!(self.env().caller() == self.owner, Error::Permission);
            require!(size > 0, Error::BetSizeTooSmall);
            self.min_bet_size = size;
            Ok(())
        }

        /// Create a new Rock-Paper-Scissors game with the amount transferred as bet size.
        ///
        /// `rounds` is the number of rounds to be played until a winner is determined.
        #[ink(message, payable)]
        pub fn create_game(&mut self, rounds: u8) -> Result<Hash> {
            let game_id = self.generate_game_id()?;
            self.games.insert(&game_id, &Game::default());
            let caller = self.env().caller();
            let bet = self.env().transferred_value();
            let block = self.env().block_number();
            let creation = Create {
                rounds,
                bet,
                min_bet_size: self.min_bet_size,
            };
            let update = Update::new(caller, block, creation);
            self.update(game_id, &update)?;
            Ok(game_id)
        }

        /// Cancel a game.
        #[ink(message)]
        pub fn cancel_game(&mut self, game_id: Hash) -> Result<()> {
            let caller = self.env().caller();
            let block = self.env().block_number();
            let update = Update::new(caller, block, Cancel {});
            self.update(game_id, &update)
        }

        /// Join a new game.
        #[ink(message, payable)]
        pub fn join_game(&mut self, game_id: Hash) -> Result<()> {
            let bet = self.env().transferred_value();
            let caller = self.env().caller();
            let block = self.env().block_number();
            let update = Update::new(caller, block, Join(bet));
            self.update(game_id, &update)
        }

        /// Enter the proof for your hand.
        #[ink(message)]
        pub fn commit_hand(&mut self, game_id: Hash, proof: Hash) -> Result<()> {
            let caller = self.env().caller();
            let block = self.env().block_number();
            let update = Update::new(caller, block, Proof(proof));
            self.update(game_id, &update)
        }

        /// Disclose your hand.
        #[ink(message)]
        pub fn disclose_hand(&mut self, game_id: Hash, hand: Hand) -> Result<()> {
            let caller = self.env().caller();
            let block = self.env().block_number();
            let update = Update::new(caller, block, hand);
            self.update(game_id, &update)
        }

        fn generate_game_id(&mut self) -> core::result::Result<Hash, Error> {
            let game_id = self.env().random(self.env().caller().as_ref()).0;
            // Is this safe from collisions?
            require!(!self.games.contains(game_id), Error::TryAgain);
            Ok(game_id)
        }

        /// Execute the update and handle potential refunds.
        ///
        /// Emits an event with the new game state.
        fn update<T>(&mut self, game_id: Hash, update: &Update<T>) -> Result<()>
        where
            Update<T>: Transition<State = Game, Error = Fail>,
        {
            let game = self
                .games
                .get(game_id)
                .ok_or(Error::InvalidGameId)?
                .step(update)
                .map_err(Error::Move)?;
            self.games.insert(&game_id, &game);
            self.refund_handler(&game)?;
            self.env().emit_event(GameEvent {
                caller: update.account,
                game_id,
                game,
            });
            Ok(())
        }

        /// Refund transfers are made for completed games.
        ///
        /// A potential winner gets the whole bet as a prize.
        /// In a draw situation, both players are refunded.
        ///
        /// If the game was cancelled, the bet goes to the account elegible for a refund.
        /// When there is no one to be refunded, the bet belongs the caller.
        ///
        /// Other game states are considered for refunding.
        fn refund_handler(&self, game: &Game) -> Result<()> {
            match game.state {
                GameState::GameOver(Outcome::Winner(Some(account))) => {
                    self.env().transfer(account, game.bet)
                }
                GameState::GameOver(Outcome::Winner(None)) => {
                    let size = game.bet / 2;
                    self.env()
                        .transfer(game.initiator.account, size)
                        .and(self.env().transfer(game.opponent.account, size))
                }
                GameState::GameOver(Outcome::Canceled(Some(account))) => {
                    self.env().transfer(account, game.bet)
                }
                GameState::GameOver(Outcome::Canceled(None)) => {
                    self.env().transfer(self.env().caller(), game.bet)
                }
                _ => Ok(()),
            }
            .map_err(|_| Error::Transfer)
        }
    }

    /// Unit tests in Rust are normally defined within such a `#[cfg(test)]`
    /// module and test functions are marked with a `#[test]` attribute.
    /// The below code is technically just normal Rust code.
    #[cfg(test)]
    mod tests {
        /// Imports all the definitions from the outer scope so we can use them here.
        use super::*;

        /// Imports `ink_lang` so we can use `#[ink::test]`.
        use ink_lang as ink;

        /// We test if the default constructor does its job.
        #[ink::test]
        fn constructor_works() {
            let rock_paper_scissors = RockPaperScissors::default();
            assert!(rock_paper_scissors.min_bet_size == 0);

            let rock_paper_scissors = RockPaperScissors::new(1234);
            assert!(rock_paper_scissors.min_bet_size == 1234);
        }

        /// We test a simple use case of our contract.
        #[ink::test]
        fn create_game_works() {
            let mut contract = RockPaperScissors::new(1234);
            let alice =
                ink_env::test::default_accounts::<ink_env::DefaultEnvironment>().alice;

            ink_env::test::set_caller::<ink_env::DefaultEnvironment>(alice);
            ink_env::test::set_value_transferred::<ink_env::DefaultEnvironment>(1234);
            let id = contract.create_game(1).unwrap();
            let game = contract.games.get(id).unwrap();
            assert!(game.bet == 1234);
            assert!(game.rounds == 1);
            assert!(game.initiator.account == alice);
        }
    }
}
