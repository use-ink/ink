use crate::{
    EventRecordOf,
    RuntimeCall,
    RuntimeEnv,
};
use frame_support::sp_runtime::{
    DispatchResultWithInfo,
    traits::{
        Dispatchable,
        Saturating,
    },
};
use frame_system::pallet_prelude::BlockNumberFor;

/// System API for the runtime.
pub trait SystemAPI {
    /// The runtime system config.
    type T: frame_system::Config;

    /// Build a new empty block and return the new height.
    fn build_block(&mut self) -> BlockNumberFor<Self::T>;

    /// Build `n` empty blocks and return the new height.
    ///
    /// # Arguments
    ///
    /// * `n` - The number of blocks to build.
    fn build_blocks(&mut self, n: u32) -> BlockNumberFor<Self::T>;

    /// Return the current height of the chain.
    fn block_number(&mut self) -> BlockNumberFor<Self::T>;

    /// Return the events of the current block so far.
    fn events(&mut self) -> Vec<EventRecordOf<Self::T>>;

    /// Reset the events of the current block.
    fn reset_events(&mut self);

    /// Execute a runtime call (dispatchable).
    ///
    /// # Arguments
    ///
    /// * `call` - The runtime call to execute.
    /// * `origin` - The origin of the call.
    fn runtime_call<Origin: Into<<RuntimeCall<Self::T> as Dispatchable>::RuntimeOrigin>>(
        &mut self,
        call: RuntimeCall<Self::T>,
        origin: Origin,
    ) -> DispatchResultWithInfo<<RuntimeCall<Self::T> as Dispatchable>::PostInfo>;
}

impl<T> SystemAPI for T
where
    T: RuntimeEnv,
    T::Runtime: frame_system::Config,
{
    type T = T::Runtime;

    fn build_block(&mut self) -> BlockNumberFor<Self::T> {
        self.execute_with(|| {
            let mut current_block = frame_system::Pallet::<Self::T>::block_number();
            let block_hash = T::finalize_block(current_block);
            current_block.saturating_inc();
            T::initialize_block(current_block, block_hash);
            current_block
        })
    }

    fn build_blocks(&mut self, n: u32) -> BlockNumberFor<Self::T> {
        let mut last_block = None;
        for _ in 0..n {
            last_block = Some(self.build_block());
        }
        last_block.unwrap_or_else(|| self.block_number())
    }

    fn block_number(&mut self) -> BlockNumberFor<Self::T> {
        self.execute_with(frame_system::Pallet::<Self::T>::block_number)
    }

    fn events(&mut self) -> Vec<EventRecordOf<Self::T>> {
        self.execute_with(frame_system::Pallet::<Self::T>::events)
    }

    fn reset_events(&mut self) {
        self.execute_with(frame_system::Pallet::<Self::T>::reset_events)
    }

    fn runtime_call<
        Origin: Into<<RuntimeCall<Self::T> as Dispatchable>::RuntimeOrigin>,
    >(
        &mut self,
        call: RuntimeCall<Self::T>,
        origin: Origin,
    ) -> DispatchResultWithInfo<<RuntimeCall<Self::T> as Dispatchable>::PostInfo> {
        self.execute_with(|| call.dispatch(origin.into()))
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        DefaultRuntime,
        RuntimeCall,
        RuntimeEnv,
        RuntimeEventOf,
        RuntimeOf,
        api::prelude::*,
    };
    use frame_support::sp_runtime::{
        AccountId32,
        DispatchResultWithInfo,
        traits::Dispatchable,
    };

    fn make_transfer(
        runtime: &mut DefaultRuntime,
        dest: AccountId32,
        value: u128,
    ) -> DispatchResultWithInfo<
        <RuntimeCall<<DefaultRuntime as RuntimeEnv>::Runtime> as Dispatchable>::PostInfo,
    > {
        assert_ne!(
            DefaultRuntime::default_actor(),
            dest,
            "make_transfer should send to account different than default_actor"
        );
        runtime.runtime_call(
            RuntimeCall::<RuntimeOf<DefaultRuntime>>::Balances(pallet_balances::Call::<
                RuntimeOf<DefaultRuntime>,
            >::transfer_allow_death {
                dest: dest.into(),
                value,
            }),
            Some(DefaultRuntime::default_actor()),
        )
    }

    #[test]
    fn dry_run_works() {
        let mut runtime = DefaultRuntime::default();
        let actor = DefaultRuntime::default_actor();
        let initial_balance = runtime.free_balance(&actor);

        runtime.dry_run(|runtime| {
            crate::api::balance_api::BalanceAPI::mint_into(runtime, &actor, 100).unwrap();
            assert_eq!(runtime.free_balance(&actor), initial_balance + 100);
        });

        assert_eq!(runtime.free_balance(&actor), initial_balance);
    }

    #[test]
    fn runtime_call_works() {
        let mut runtime = DefaultRuntime::default();

        const RECIPIENT: AccountId32 = AccountId32::new([2u8; 32]);
        let initial_balance = runtime.free_balance(&RECIPIENT);

        let result = make_transfer(&mut runtime, RECIPIENT, 100);
        assert!(result.is_ok());

        let expected_balance = initial_balance + 100;
        assert_eq!(runtime.free_balance(&RECIPIENT), expected_balance);
    }

    #[test]
    fn current_events() {
        let mut runtime = DefaultRuntime::default();
        const RECIPIENT: AccountId32 = AccountId32::new([2u8; 32]);

        let events_before = runtime.events();
        assert!(events_before.is_empty());

        make_transfer(&mut runtime, RECIPIENT, 1).expect("Failed to make transfer");

        let events_after = runtime.events();
        assert!(!events_after.is_empty());
        assert!(matches!(
            events_after.last().unwrap().event,
            RuntimeEventOf::<DefaultRuntime>::Balances(_)
        ));
    }

    #[test]
    fn resetting_events() {
        let mut runtime = DefaultRuntime::default();
        const RECIPIENT: AccountId32 = AccountId32::new([3u8; 32]);

        make_transfer(&mut runtime, RECIPIENT.clone(), 1)
            .expect("Failed to make transfer");

        assert!(!runtime.events().is_empty());
        runtime.reset_events();
        assert!(runtime.events().is_empty());

        make_transfer(&mut runtime, RECIPIENT, 1).expect("Failed to make transfer");
        assert!(!runtime.events().is_empty());
    }

    #[test]
    fn snapshot_works() {
        let mut runtime = DefaultRuntime::default();

        // Check state before
        let block_before = runtime.block_number();
        let snapshot_before = runtime.take_snapshot();

        // Advance some blocks to have some state change
        let _ = runtime.build_blocks(5);
        let block_after = runtime.block_number();

        // Check block number and state after
        assert_eq!(block_before + 5, block_after);

        // Restore state
        runtime.restore_snapshot(snapshot_before);

        // Check state after restore
        assert_eq!(block_before, runtime.block_number());
    }
}
