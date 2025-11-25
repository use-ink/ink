# Assets Precompile Integration Test (Ethereum-First Approach)

This integration test demonstrates the **recommended approach** for interacting with pallet-revive contracts using Ethereum addresses as the default.

## Why Ethereum Addresses?

When using Substrate addresses (AccountId32) with pallet-revive, you encounter the "fallback account trap":

```
Substrate AccountId32 (Alice)
    │
    ▼ keccak_256()[12..]
Derived H160 (0xabcd...)
    │
    ▼ Contract stores this address
    │
    ▼ Later: Contract needs to send tokens back
    │
    ▼ to_account_id(0xabcd...) lookup
    │
    ├─ IF MAPPED: Returns original AccountId32 ✓
    │
    └─ IF NOT MAPPED: Returns FALLBACK account ✗
                      (0xabcd...EEEEEEEEEEEEEEEEEEEE)
                      Tokens are effectively lost!
```

## The Ethereum-First Solution

By using Ethereum keypairs (ECDSA/secp256k1), the address roundtrip is **lossless**:

```
Ethereum H160 (0x1234...)
    │
    ▼ to_account_id()
Fallback AccountId32 (0x1234...EEEEEEEEEEEEEEEEEEEE)
    │
    ▼ to_address() checks is_eth_derived()
    │
    ▼ TRUE → strips 0xEE suffix
    │
Original H160 (0x1234...) ✓
```

## Key Differences from Standard Approach

| Aspect | Substrate Approach | Ethereum Approach |
|--------|-------------------|-------------------|
| Keypair type | Sr25519 (`alice()`, `bob()`) | ECDSA (`alith()`, `baltathar()`) |
| Address type | Derived H160 | Native H160 |
| Mapping required | Yes (`map_account()`) | **No!** |
| Address roundtrip | Lossy without mapping | Lossless |
| MetaMask compatible | No | Yes |

## Usage

```rust
use ink_e2e::eth::{alith, baltathar};

// Use Ethereum keypairs
let alice = alith();
let bob = baltathar();

// Get native H160 addresses
let alice_address = alice.address();
let bob_address = bob.address();

// No mapping needed - just use them directly with contracts!
```

## Available Dev Accounts

The following Ethereum dev accounts are available via `ink_e2e::eth::dev`:

- `alith()` - Primary test account
- `baltathar()` - Secondary test account
- `charleth()` - Third test account
- `dorothy()` - Fourth test account
- `ethan()` - Fifth test account
- `faith()` - Sixth test account
- ... and more

## Running the Tests

```bash
cargo test -p assets_precompile_eth --features e2e-tests
```
