# Proposal: Expansion of ink! Example Repository

This proposal outlines a set of new, high-impact examples to be added to the `integration-tests/public` directory. These additions aim to demonstrate real-world use cases, advanced patterns, and emerging standards in the Polkadot ecosystem.

## 1. Decentralized Finance (DeFi)
*DeFi primitives are the backbone of most blockchain ecosystems. These examples will show how to handle complex math and asset management.*

| Example | Directory | Use Case |
| :--- | :--- | :--- |
| **AMM / CPMM** | `defi/amm` | A Constant Product Market Maker (like Uniswap v2). Demonstrates token swapping, liquidity provision, and price calculation logic. |
| **Staking Rewards** | `defi/staking` | A contract where users lock tokens to earn rewards over time. Demonstrates time-based logic and reward calculation. |
| **Wrapped Token** | `defi/wrapper` | Logic to wrap the native chain currency (e.g., DOT/AZERO) into a fungible token standard (PSP22/ERC20). |
| **Lending Pool** | `defi/lending` | A simplified lending protocol allowing users to supply assets for interest and borrow against collateral. |

## 2. Governance (DAO)
* Decentralized governance is critical for protocol management.*

| Example | Directory | Use Case |
| :--- | :--- | :--- |
| **Simple DAO** | `governance/dao` | Proposal creation and voting mechanism. Demonstrates struct storage for proposals and voting power verification. |
| **Timelock** | `governance/timelock` | A controller that delays transaction execution for a set period. Critical for security in admin-controlled contracts. |
| **Quadratic Voting** | `governance/quadratic` | A voting system where the cost of a vote increases quadratically. Demonstrates complex math and sybil-resistance usage. |

## 3. Design Patterns & Security
*Standard, reusable patterns for secure contract development.*

| Example | Directory | Use Case |
| :--- | :--- | :--- |
| **RBAC (Access Control)**| `patterns/rbac` | Role-Based Access Control. A flexible system for assigning roles (Admin, Minter, Burner) rather than just a simplistic `OnlyOwner`. |
| **Pausable** | `patterns/pausable` | An emergency stop mechanism. Allows admins to freeze contract functionality during incidents. |
| **Pull vs Push** | `patterns/payment-splitter` | Demonstrates the "Pull over Push" payment pattern to avoid reentrancy and stuck funds when distributing value to multiple recipients. |
| **Proxy Decorator** | `patterns/proxy` | A delegate proxy pattern that forwards calls to an implementation contract (if not fully covered by `upgradeable`). |

## 4. Identity & Social
*Web3 social and identity primitives.*

| Example | Directory | Use Case |
| :--- | :--- | :--- |
| **DID Registry** | `identity/did` | A Decentralized Identifier registry managing attributes and delegates for identities. |
| **Social Graph** | `social/graph` | Managing follower/following relationships on-chain. Demonstrates efficient storage mapping for graph data. |

## 5. Gaming & NFTs
*Advanced NFT usage beyond simple minting.*

| Example | Directory | Use Case |
| :--- | :--- | :--- |
| **NFT Breeding** | `gaming/breeding` | Combining two NFTs to produce a third with mixed attributes. Demonstrates pseudo-randomness and attribute logic. |
| **Gacha / Lootbox** | `gaming/lootbox` | Minting random items with rarity tiers. Focuses on randomness and probability distribution. |

## Implementation Roadmap

We recommend implementing these in phases:
1.  **Phase 1 (Core)**: RBAC, AMM, and Simple DAO.
2.  **Phase 2 (Utility)**: Wrapped Token, Staking, and Pausable.
3.  **Phase 3 (Verticals)**: Gaming and Identity examples.
