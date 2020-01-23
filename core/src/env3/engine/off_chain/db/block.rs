use super::super::{
    types::{
        OffBlockNumber,
        OffMoment,
    },
    TypedEncoded,
    Result,
};
use crate::env3::EnvTypes;

/// An emulated block in the chain.
pub struct Block {
    /// The current block number.
    number: OffBlockNumber,
    /// The current moment of block creation.
    moment: OffMoment,
}

impl Block {
    /// Creates a new block for the given number and moment.
    pub fn new<T>(number: T::BlockNumber, moment: T::Moment) -> Self
    where
        T: EnvTypes,
    {
        Self {
            number: TypedEncoded::new(&number),
            moment: TypedEncoded::new(&moment),
        }
    }

    /// Returns the block number.
    pub fn number<T>(&self) -> Result<T::BlockNumber>
    where
        T: EnvTypes,
    {
        self.number.decode().map_err(Into::into)
    }

    /// Returns the moment of the block.
    pub fn moment<T>(&self) -> Result<T::Moment>
    where
        T: EnvTypes,
    {
        self.moment.decode().map_err(Into::into)
    }
}
