use drink::session::Session;
use drink::runtime::MinimalRuntime;

pub struct Client {
    session: Session<MinimalRuntime>,
}
