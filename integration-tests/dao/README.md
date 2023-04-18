# DAO
A very simple DAO. 

## How to use?
- Instantiate ERC20 contract.
- Instantiate DAO contract with instantiated ERC20 contract.
- Have fun!

## Functionality
- Make a proposal
- Vote on a proposal
- Execute a proposal
- Fund the treasury

## Rules
- Upon **DAO** creation the governance token and quorum is specified.
- **Voting power** is decided and can be modified in `fn get_voting_power`.
- **Governance token**, upon dao creation, gives accounts voting power.
- **Payout token** is the native token and held by the treasury.
- **Treasury** can be funded upon dao creation (`fn new`) and through `fn fund`.

## Ideas:
### DAO:
- Upon DAO creation:
  * Constant proposal duration.
  * Minimum proposal duration.
  * Maximum allowed proposal duration.
 
### Voting power:
- Different mechanics for voting power (e.g. quadratic voting).
- Voter can specify how much it wants to vote.

### Governance token:
- Native token.
- Same as the payout token.

### Payout token:
- Erc20 token.
- Same as the governance token.
- Multiple tokens, can be specified by the proposer.

## Details
- ProposalId -> Hash?
- How to make a DAO unique (searchable)?
  * Name
  * Name + token
  * Name + token + blocknumber
- Who should be able to execute a proposal?
- Should we be able to query who voted on a proposal?
  * BoundedVec<AccountId>
- How to check whether the DAO has enough funds when there are ongoing proposals
  that can empty the treasury?
