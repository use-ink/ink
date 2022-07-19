use super::require;
use core::cmp::Ordering;
use ink_env::{
    hash::{
        HashOutput,
        Sha2x256,
    },
    AccountId,
    Hash,
};
use ink_primitives::KeyPtr;
use ink_storage::traits::{
    PackedLayout,
    SpreadAllocate,
    SpreadLayout,
};
use scale::{
    Decode,
    Encode,
};

type BlockNo = u32;

/// How many blocks may pass until the game is considered to be timed out.
pub(crate) const TIMEOUT: BlockNo = 100;

// TODO check are all these needed
#[derive(Debug, PartialEq, Eq, Encode, Decode)]
#[cfg_attr(
    feature = "std",
    derive(::scale_info::TypeInfo, ink_storage::traits::StorageLayout,)
)]
pub enum Fail {
    BetSizeTooSmall,
    WrongAccount,
    WrongGameState,
    NotYet,
    OnlyOnce,
    InvalidHand,
    InvalidConfiguration,
    Timeout,
}

#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode, SpreadLayout, PackedLayout)]
#[cfg_attr(
    feature = "std",
    derive(::scale_info::TypeInfo, ink_storage::traits::StorageLayout,)
)]
pub enum Outcome {
    Winner(Option<AccountId>),
    Canceled(Option<AccountId>),
}

#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode, SpreadLayout, PackedLayout)]
#[cfg_attr(
    feature = "std",
    derive(::scale_info::TypeInfo, ink_storage::traits::StorageLayout,)
)]
pub enum GameState {
    Constructing,
    LookingForOpponent(BlockNo),
    Proofing(BlockNo),
    Disclosing(BlockNo),
    GameOver(Outcome),
}

impl Default for GameState {
    fn default() -> Self {
        GameState::Constructing
    }
}

impl SpreadAllocate for GameState {
    #[inline]
    fn allocate_spread(ptr: &mut KeyPtr) -> Self {
        ptr.advance_by(<Option<AccountId>>::FOOTPRINT * 3);
        Self::Constructing
    }
}

#[derive(
    Default,
    Debug,
    PartialEq,
    Clone,
    Encode,
    Decode,
    SpreadLayout,
    PackedLayout,
    SpreadAllocate,
)]
#[cfg_attr(
    feature = "std",
    derive(scale_info::TypeInfo, ink_storage::traits::StorageLayout,)
)]
pub(crate) struct Player {
    pub(crate) account: AccountId,
    pub(crate) proof: Option<Hash>,
    pub(crate) hand: Option<Hand>,
    pub(crate) wins: u8,
}

impl Player {
    pub(crate) fn new(account: AccountId) -> Self {
        Self {
            account,
            ..Default::default()
        }
    }

    /// Reset the players proof and hand for the next round.
    fn reset(&mut self) {
        self.proof = None;
        self.hand = None;
    }
}

/// Interface allowing updates to change the game state.
///
/// The split between validate and transition is to have adedicated place for any checks.
/// This should make it easier to reason about conditions the update can be applied successfully.
pub(crate) trait Transition {
    type State;
    type Error;

    /// This function should check whether the update is allowed to trigger a state transition.
    fn validate(&self, state: &Self::State) -> Result<(), Self::Error>;

    /// Applies the update to the curent `state`.
    fn transition(&self, state: Self::State) -> Self::State;
}

pub(crate) struct Update<T> {
    pub(crate) account: AccountId,
    pub(crate) block: BlockNo,
    pub(crate) message: T,
}

impl<T> Update<T> {
    pub(crate) fn new(account: AccountId, block: BlockNo, message: T) -> Self {
        Self {
            account,
            block,
            message,
        }
    }
}

pub(crate) struct Create {
    pub(crate) rounds: u8,
    pub(crate) bet: u128,
    pub(crate) min_bet_size: u128,
}

impl Transition for Update<Create> {
    type State = Game;
    type Error = Fail;

    fn validate(&self, game: &Game) -> Result<(), Fail> {
        require!(
            matches!(game.state, GameState::Constructing),
            Fail::WrongGameState
        );
        require!(self.message.rounds > 0, Fail::InvalidConfiguration);
        require!(
            self.message.bet >= self.message.min_bet_size,
            Fail::BetSizeTooSmall
        );
        Ok(())
    }

    fn transition(&self, mut game: Game) -> Game {
        game.rounds = self.message.rounds;
        game.initiator = Player::new(self.account);
        game.bet = self.message.bet;
        game.state = GameState::LookingForOpponent(self.block);
        game
    }
}

pub(crate) struct Cancel;

/// Cancellation is allowed when the game timed out (and wasn't finished).
/// If it hasn-t started yet, only the initiator can cancel.
/// Otherwise anyone can cancel games.
impl Transition for Update<Cancel> {
    type State = Game;
    type Error = Fail;

    fn validate(&self, game: &Game) -> Result<(), Fail> {
        require!(game.timeout(self.block), Fail::NotYet);

        match game.state {
            GameState::GameOver(_) => return Err(Fail::WrongGameState),
            GameState::LookingForOpponent(_) => {
                require!(game.initiator.account == self.account, Fail::WrongAccount)
            }
            GameState::Constructing
            | GameState::Proofing(_)
            | GameState::Disclosing(_) => {}
        };
        Ok(())
    }

    fn transition(&self, mut game: Game) -> Game {
        game.state = match game.state {
            GameState::Constructing => GameState::GameOver(Outcome::Canceled(None)),
            GameState::LookingForOpponent(_) => {
                GameState::GameOver(Outcome::Canceled(Some(self.account)))
            }
            GameState::Proofing(_) => {
                let complied = match (game.initiator.proof, game.opponent.proof) {
                    (Some(_), None) => Some(game.initiator.account),
                    (None, Some(_)) => Some(game.opponent.account),
                    _ => None,
                };
                GameState::GameOver(Outcome::Canceled(complied))
            }
            GameState::Disclosing(_) => {
                let complied = match (&game.initiator.hand, &game.opponent.hand) {
                    (Some(_), None) => Some(game.initiator.account),
                    (None, Some(_)) => Some(game.opponent.account),
                    _ => None,
                };
                GameState::GameOver(Outcome::Canceled(complied))
            }
            _ => return game,
        };
        game
    }
}

/// Security: Balances and validation should happen outside
pub(crate) struct Join(pub(crate) u128);

impl Transition for Update<Join> {
    type State = Game;
    type Error = Fail;

    fn validate(&self, game: &Game) -> Result<(), Fail> {
        require!(
            matches!(game.state, GameState::LookingForOpponent(_)),
            Fail::WrongGameState
        );
        require!(game.initiator.account != self.account, Fail::WrongAccount);
        require!(self.message.0 >= game.bet, Fail::BetSizeTooSmall);
        Ok(())
    }

    fn transition(&self, mut game: Game) -> Game {
        game.opponent = Player::new(self.account);
        game.bet += self.message.0;
        game.state = GameState::Proofing(self.block);
        game
    }
}

pub struct Proof(pub(crate) Hash);

impl Transition for Update<Proof> {
    type State = Game;
    type Error = Fail;

    fn validate(&self, game: &Game) -> Result<(), Fail> {
        require!(
            matches!(game.state, GameState::Proofing(_)),
            Fail::WrongGameState
        );
        require!(!game.timeout(self.block), Fail::Timeout);
        require!(game.is_player(self.account), Fail::WrongAccount);
        if game.is_initiator(self.account) {
            require!(game.initiator.proof.is_none(), Fail::OnlyOnce);
        } else {
            require!(game.opponent.proof.is_none(), Fail::OnlyOnce);
        }
        Ok(())
    }

    fn transition(&self, mut game: Game) -> Game {
        if game.is_initiator(self.account) {
            game.initiator.proof = Some(self.message.0)
        } else {
            game.opponent.proof = Some(self.message.0);
        }

        if game.initiator.proof.is_some() && game.opponent.proof.is_some() {
            game.state = GameState::Disclosing(self.block)
        }
        game
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode, SpreadLayout, PackedLayout)]
#[cfg_attr(
    feature = "std",
    derive(::scale_info::TypeInfo, ink_storage::traits::StorageLayout,)
)]
pub enum Hand {
    Rock(u128),
    Paper(u128),
    Scissors(u128),
}

impl Hand {
    pub(crate) fn proof(&self) -> Hash {
        let token = match self {
            Hand::Rock(nonce) => nonce ^ 1,
            Hand::Paper(nonce) => nonce ^ 2,
            Hand::Scissors(nonce) => nonce ^ 3,
        };
        let mut output = <Sha2x256 as HashOutput>::Type::default();
        ink_env::hash_bytes::<Sha2x256>(&token.to_be_bytes()[..], &mut output);
        output.into()
    }
}

impl Transition for Update<Hand> {
    type State = Game;
    type Error = Fail;

    fn validate(&self, game: &Game) -> Result<(), Fail> {
        require!(
            matches!(game.state, GameState::Disclosing(_)),
            Fail::WrongGameState
        );
        require!(!game.timeout(self.block), Fail::Timeout);
        require!(game.is_player(self.account), Fail::WrongAccount);
        let player = if game.is_initiator(self.account) {
            &game.initiator
        } else {
            &game.opponent
        };
        require!(player.hand.is_none(), Fail::OnlyOnce);
        require!(
            player.proof.unwrap() == self.message.proof(),
            Fail::InvalidHand
        );

        Ok(())
    }

    fn transition(&self, mut game: Game) -> Game {
        if game.is_initiator(self.account) {
            game.initiator.hand = Some(self.message.clone())
        } else {
            game.opponent.hand = Some(self.message.clone())
        }
        if game.initiator.hand.is_none() || game.opponent.hand.is_none() {
            return game
        }

        match (
            game.initiator.hand.as_ref().unwrap(),
            game.opponent.hand.as_ref().unwrap(),
        ) {
            (Hand::Rock(_), Hand::Scissors(_))
            | (Hand::Paper(_), Hand::Rock(_))
            | (Hand::Scissors(_), Hand::Paper(_)) => game.initiator.wins += 1,
            (Hand::Rock(_), Hand::Paper(_))
            | (Hand::Paper(_), Hand::Scissors(_))
            | (Hand::Scissors(_), Hand::Rock(_)) => game.opponent.wins += 1,
            _ => {}
        };

        game.rounds -= 1;
        if game.rounds == 0 {
            let winner = match game.initiator.wins.cmp(&game.opponent.wins) {
                Ordering::Less => Some(game.opponent.account),
                Ordering::Equal => None,
                Ordering::Greater => Some(game.initiator.account),
            };
            game.state = GameState::GameOver(Outcome::Winner(winner));
            return game
        }

        game.initiator.reset();
        game.opponent.reset();
        game.state = GameState::Proofing(self.block);
        game
    }
}

#[derive(
    Default,
    scale::Encode,
    scale::Decode,
    PartialEq,
    Debug,
    Clone,
    SpreadLayout,
    PackedLayout,
    SpreadAllocate,
)]
#[cfg_attr(
    feature = "std",
    derive(scale_info::TypeInfo, ink_storage::traits::StorageLayout,)
)]
pub(crate) struct Game {
    pub(crate) rounds: u8,
    pub(crate) state: GameState,
    pub(crate) initiator: Player,
    pub(crate) opponent: Player,
    pub(crate) bet: u128,
}

impl Game {
    pub(crate) fn step<T>(self, message: &T) -> Result<Self, Fail>
    where
        T: Transition<State = Self, Error = Fail>,
    {
        message.validate(&self)?;
        Ok(message.transition(self))
    }

    fn timeout(&self, current: BlockNo) -> bool {
        match self.state {
            GameState::LookingForOpponent(last)
            | GameState::Proofing(last)
            | GameState::Disclosing(last) => last + TIMEOUT <= current,
            _ => false,
        }
    }

    fn is_initiator(&self, account: AccountId) -> bool {
        self.initiator.account == account
    }

    fn is_opponent(&self, account: AccountId) -> bool {
        self.opponent.account == account
    }

    fn is_player(&self, account: AccountId) -> bool {
        self.is_initiator(account) || self.is_opponent(account)
    }
}

/// Unit tests for all update messages belong here
#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn timeout_works_correct() {
        let mut g = Game::default();
        assert!(!g.timeout(0));

        g.state = GameState::LookingForOpponent(0);
        assert!(!g.timeout(0));
        assert!(!g.timeout(TIMEOUT - 1));
        assert!(g.timeout(TIMEOUT));
        assert!(g.timeout(0 + TIMEOUT));
    }

    #[test]
    fn create_message_can_transition() {
        let mut g = Game::default();
        let mut update = Update {
            account: AccountId::from([0x01; 32]),
            block: 1,
            message: Create {
                rounds: 1,
                bet: 1,
                min_bet_size: 1,
            },
        };

        // Invalid rounds
        update.message.rounds = 0;
        assert_eq!(update.validate(&g), Err(Fail::InvalidConfiguration));
        update.message.rounds = 1;

        // Bet too small
        update.message.bet = 0;
        assert_eq!(update.validate(&g), Err(Fail::BetSizeTooSmall));
        update.message.bet = 1;

        // Expect successful transition
        g = update.transition(g);
        assert_eq!(g.state, GameState::LookingForOpponent(update.block));

        // Expect wrong GameState now
        assert_eq!(update.validate(&g), Err(Fail::WrongGameState));
    }
}
