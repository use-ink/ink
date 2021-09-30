// Copyright 2018-2021 Parity Technologies (UK) Ltd.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use derive_more::Display;

/// A dispatch error.
#[derive(Debug, Copy, Clone, Display)]
pub enum DispatchError {
    #[display(fmt = "unknown selector")]
    UnknownSelector,
    #[display(fmt = "unknown constructor selector")]
    UnknownInstantiateSelector,
    #[display(fmt = "unknown message selector")]
    UnknownCallSelector,

    #[display(fmt = "unable to decoded input parameter bytes")]
    InvalidParameters,
    #[display(fmt = "unable to decoded input parameter bytes for constructor")]
    InvalidInstantiateParameters,
    #[display(fmt = "unable to decoded input parameter bytes for message")]
    InvalidCallParameters,

    #[display(fmt = "could not read input parameters")]
    CouldNotReadInput,
    #[display(fmt = "paid an unpayable message")]
    PaidUnpayableMessage,
}
