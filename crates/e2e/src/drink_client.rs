use crate::ChainBackend;
use drink::{
    chain_api::ChainApi,
    runtime::MinimalRuntime,
    session::Session,
};
use jsonrpsee::core::async_trait;
use sp_core::crypto::AccountId32;
use sp_core::Pair;
use subxt::dynamic::Value;

pub struct Client {
    session: Session<MinimalRuntime>,
}

unsafe impl Send for Client {}
unsafe impl Sync for Client {}

#[async_trait]
impl ChainBackend for Client {
    type Actor = AccountId32;
    type ActorId = AccountId32;
    type Balance = u128;
    type Error = ();
    type EventLog = ();

    async fn create_and_fund_account(
        &mut self,
        _origin: &Self::Actor,
        amount: Self::Balance,
    ) -> Self::Actor {
        let (pair, _) = <sp_core::sr25519::Pair as sp_core::Pair>::generate();
        let new_account = AccountId32::new(pair.public().0);

        self.session
            .chain_api()
            .add_tokens(new_account.clone(), amount);
        new_account
    }

    async fn balance(&mut self, actor: Self::ActorId) -> Result<Self::Balance, Self::Error> {
        Ok(self.session.chain_api().balance(&actor))
    }

    async fn runtime_call<'a>(
        &mut self,
        _actor: &Self::Actor,
        _pallet_name: &'a str,
        _call_name: &'a str,
        _call_data: Vec<Value>,
    ) -> Result<Self::EventLog, Self::Error> {
        todo!()
    }
}
