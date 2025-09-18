use crate::Sandbox;

/// Generic Time type.
type MomentOf<R> = <R as pallet_timestamp::Config>::Moment;

/// Timestamp API used to interact with the timestamp pallet.
pub trait TimestampAPI {
    /// The runtime timestamp config.
    type T: pallet_timestamp::Config;

    /// Return the timestamp of the current block.
    fn get_timestamp(&mut self) -> MomentOf<Self::T>;

    /// Set the timestamp of the current block.
    ///
    /// # Arguments
    ///
    /// * `timestamp` - The new timestamp to be set.
    fn set_timestamp(&mut self, timestamp: MomentOf<Self::T>);
}

impl<T> TimestampAPI for T
where
    T: Sandbox,
    T::Runtime: pallet_timestamp::Config,
{
    type T = T::Runtime;

    fn get_timestamp(&mut self) -> MomentOf<Self::T> {
        self.execute_with(pallet_timestamp::Pallet::<T::Runtime>::get)
    }

    fn set_timestamp(&mut self, timestamp: MomentOf<Self::T>) {
        self.execute_with(|| {
            pallet_timestamp::Pallet::<T::Runtime>::set_timestamp(timestamp)
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        DefaultSandbox,
        api::prelude::*,
    };

    #[test]
    fn getting_and_setting_timestamp_works() {
        let mut sandbox = DefaultSandbox::default();
        for timestamp in 0..10 {
            assert_ne!(sandbox.get_timestamp(), timestamp);
            sandbox.set_timestamp(timestamp);
            assert_eq!(sandbox.get_timestamp(), timestamp);

            sandbox.build_block();
        }
    }
}
