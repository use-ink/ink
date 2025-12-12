//! # NFTs Call Runtime Example
//!
//! This contract demonstrates how to call `pallet-nfts` runtime functions from
//! an ink! contract using `xcm_execute` with the `Transact` instruction.
//!
//! ## How It Works
//!
//! The contract uses `xcm_execute` with a `Transact` instruction to dispatch
//! calls to `pallet-nfts`. The call arguments are passed as pre-encoded bytes,
//! keeping the contract minimal while the caller (e.g., e2e tests) handles encoding.
//!
//! ## Error Handling Limitation
//!
//! **Important**: Specific pallet errors (e.g., `UnknownItem`, `NotOwner`) are NOT
//! propagated through the XCM error path. The contract only knows if the call
//! succeeded or failed, not WHY it failed. Use e2e tests with `RuntimeAPI` to
//! verify specific pallet errors during development.

#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod nfts_call_runtime {
    use ink::{
        prelude::vec::Vec,
        xcm::{
            DoubleEncoded,
            prelude::*,
        },
    };

    pub type CollectionId = u32;
    pub type ItemId = u32;

    // Pallet and call indices in the paseo asset hub runtime.
    // Find these by inspecting the runtime's `construct_runtime!` macro.
    const NFTS_PALLET_INDEX: u8 = 6;
    const CREATE_COLLECTION: u8 = 0;
    const MINT: u8 = 3;
    const BURN: u8 = 5;
    const TRANSFER: u8 = 6;

    /// Error type for runtime call failures.
    #[ink::error]
    pub enum RuntimeCallError {
        /// Failed to weigh the XCM message.
        WeighFailed,
        /// The runtime call failed (XCM execution error).
        ExecuteFailed,
        /// Caller is not the contract owner.
        NotOwner,
    }

    #[ink(storage)]
    pub struct NftsRuntimeCaller {
        /// Trusted collection id supplied by the deployer. The contract cannot verify
        /// that the encoded `create` call actually creates this id; keep the two
        /// in sync.
        collection: CollectionId,
        next_item: ItemId,
        owner: Address,
    }

    impl NftsRuntimeCaller {
        /// Creates a collection in the runtime.
        ///
        /// The `encoded_call_args` should be SCALE-encoded arguments for
        /// `pallet_nfts::Call::create { admin, config }`.
        /// The caller must ensure `collection` matches the id the runtime will
        /// assign; the contract cannot validate this.
        ///
        /// The constructor is payable because the runtime will reserve deposits for
        /// the collection on the contract account.
        #[ink(constructor, payable)]
        pub fn new(
            collection: CollectionId,
            encoded_call_args: Vec<u8>,
        ) -> Result<Self, RuntimeCallError> {
            let instance = Self {
                collection,
                next_item: 0,
                owner: Self::env().caller(),
            };
            instance.call_runtime(CREATE_COLLECTION, encoded_call_args)?;
            Ok(instance)
        }

        /// Returns the configured collection id.
        #[ink(message)]
        pub fn collection_id(&self) -> CollectionId {
            self.collection
        }

        /// Returns the owner of this contract.
        #[ink(message)]
        pub fn owner(&self) -> Address {
            self.owner
        }

        /// Returns the next item id that will be minted.
        #[ink(message)]
        pub fn next_item_id(&self) -> ItemId {
            self.next_item
        }

        /// Mints an NFT into the runtime using `pallet-nfts`.
        ///
        /// The `encoded_call_args` should be SCALE-encoded arguments for
        /// `pallet_nfts::Call::mint { collection, item, mint_to, witness_data }`.
        ///
        /// The mint is payable to cover item deposits in the runtime.
        #[ink(message, payable)]
        pub fn mint(
            &mut self,
            encoded_call_args: Vec<u8>,
        ) -> Result<ItemId, RuntimeCallError> {
            self.ensure_owner()?;
            let item = self.next_item;
            self.call_runtime(MINT, encoded_call_args)?;
            self.next_item = self.next_item.saturating_add(1);
            Ok(item)
        }

        /// Transfers an item owned by the contract to another account.
        ///
        /// The `encoded_call_args` should be SCALE-encoded arguments for
        /// `pallet_nfts::Call::transfer { collection, item, dest }`.
        #[ink(message)]
        pub fn transfer(
            &mut self,
            encoded_call_args: Vec<u8>,
        ) -> Result<(), RuntimeCallError> {
            self.ensure_owner()?;
            self.call_runtime(TRANSFER, encoded_call_args)
        }

        /// Burns an item in the runtime.
        ///
        /// The `encoded_call_args` should be SCALE-encoded arguments for
        /// `pallet_nfts::Call::burn { collection, item }`.
        #[ink(message)]
        pub fn burn(
            &mut self,
            encoded_call_args: Vec<u8>,
        ) -> Result<(), RuntimeCallError> {
            self.ensure_owner()?;
            self.call_runtime(BURN, encoded_call_args)
        }

        /// Execute a runtime call to pallet-nfts via XCM Transact.
        ///
        /// This builds an XCM message with:
        /// 1. `Transact` - executes the SCALE-encoded pallet call
        /// 2. `ExpectTransactStatus(Success)` - fails if the pallet call fails
        fn call_runtime(
            &self,
            call_index: u8,
            encoded_args: Vec<u8>,
        ) -> Result<(), RuntimeCallError> {
            // Build the full call: [pallet_index, call_index, encoded_args...]
            let mut encoded_call = Vec::with_capacity(2 + encoded_args.len());
            encoded_call.push(NFTS_PALLET_INDEX);
            encoded_call.push(call_index);
            encoded_call.extend(encoded_args);

            // Build XCM message with Transact + ExpectTransactStatus
            let call: DoubleEncoded<()> = encoded_call.into();
            let xcm_msg: Xcm<()> = Xcm::builder_unsafe()
                .transact(
                    OriginKind::SovereignAccount,
                    Weight::from_parts(u64::MAX, u64::MAX),
                    call,
                )
                .expect_transact_status(MaybeErrorCode::Success)
                .build();

            // Weigh and execute
            let versioned_msg = VersionedXcm::from(xcm_msg);
            let weight = self
                .env()
                .xcm_weigh(&versioned_msg)
                .map_err(|_| RuntimeCallError::WeighFailed)?;
            self.env()
                .xcm_execute(&versioned_msg, weight)
                .map_err(|_| RuntimeCallError::ExecuteFailed)
        }

        fn ensure_owner(&self) -> Result<(), RuntimeCallError> {
            if self.env().caller() != self.owner {
                return Err(RuntimeCallError::NotOwner);
            }
            Ok(())
        }
    }

    #[cfg(all(test, feature = "e2e-tests"))]
    mod e2e_tests {
        use super::*;
        use ink::scale::Encode;
        use ink_e2e::{
            ContractsBackend,
            alice,
            bob,
        };
        use ink_runtime::{
            AccountId32,
            IntoAccountId,
            api::{
                prelude::NftsAPI,
                revive_api::ContractAPI,
            },
            assert_ok,
        };
        use pallet_nfts::{
            CollectionConfig,
            CollectionSettings,
            MintSettings,
            MintType,
            MintWitness,
        };

        type E2EResult<T> = std::result::Result<T, ink_runtime::E2EError>;
        type Balance = u128;
        type BlockNumber = u32;
        type RuntimeCollectionConfig =
            CollectionConfig<Balance, BlockNumber, CollectionId>;
        type RuntimeMintSettings = MintSettings<Balance, BlockNumber, CollectionId>;
        type RuntimeMintWitness = MintWitness<ItemId, Balance>;

        /// Deployment creates a collection and honors config (e.g., max supply).
        #[ink_e2e::test(runtime)]
        async fn constructor_creates_collection_with_config(
            mut client: Client,
        ) -> E2EResult<()> {
            let admin = alice().into_account_id();
            let max_supply = Some(42u32);
            let create_args =
                encode_create_args(admin, default_collection_config(max_supply));
            let mut constructor = NftsRuntimeCallerRef::new(0, create_args);

            let contract = client
                .instantiate("nfts_call_runtime", &alice(), &mut constructor)
                .value(1_000_000_000_000u128)
                .submit()
                .await?;

            let calls = contract.call_builder::<NftsRuntimeCaller>();

            let collection_id = client
                .call(&alice(), &calls.collection_id())
                .dry_run()
                .await?;
            assert_eq!(collection_id.return_value(), 0);

            let next_item = client
                .call(&alice(), &calls.next_item_id())
                .dry_run()
                .await?;
            assert_eq!(next_item.return_value(), 0);

            let config = client
                .runtime()
                .collection_config(&0u32)
                .expect("collection config should exist");
            assert_eq!(config.max_supply, max_supply);
            assert!(client.runtime().collection_exists(&0u32));

            Ok(())
        }

        /// Happy path that mints, transfers, and checks runtime state.
        #[ink_e2e::test(runtime)]
        async fn mint_and_transfer_happy_path(mut client: Client) -> E2EResult<()> {
            let admin = alice().into_account_id();
            let create_args = encode_create_args(admin, default_collection_config(None));
            let mut constructor = NftsRuntimeCallerRef::new(0, create_args);

            let contract = client
                .instantiate("nfts_call_runtime", &alice(), &mut constructor)
                .value(1_000_000_000_000u128)
                .submit()
                .await?;

            let _ = client.runtime().map_account(&bob());
            let bob_account = bob().into_account_id();
            let mut calls = contract.call_builder::<NftsRuntimeCaller>();

            // Mint item 0 to the contract
            let mint_args =
                encode_mint_args(0, 0, to_runtime_account_id(contract.account_id));
            let mint0 = client
                .call(&alice(), &calls.mint(mint_args))
                .submit()
                .await?;
            assert_ok!(mint0);

            // Mint item 1 to bob
            let mint_args = encode_mint_args(0, 1, bob_account.clone());
            let mint1 = client
                .call(&alice(), &calls.mint(mint_args))
                .submit()
                .await?;
            assert_ok!(mint1);

            // Transfer item 0 to bob
            let transfer_args = encode_transfer_args(0, 0, bob_account.clone());
            let transfer = client
                .call(&alice(), &calls.transfer(transfer_args))
                .submit()
                .await?;
            assert_ok!(transfer);

            // Runtime-side assertions
            assert_eq!(
                client.runtime().owner_of(&0u32, &0u32),
                Some(bob_account.clone())
            );
            assert_eq!(
                client.runtime().owner_of(&0u32, &1u32),
                Some(bob_account.clone())
            );
            assert!(client.runtime().item_exists(&0u32, &0u32));
            assert!(client.runtime().item_exists(&0u32, &1u32));

            let details = client
                .runtime()
                .collection_details(&0u32)
                .expect("collection should exist");
            assert_eq!(details.items, 2);

            let next_item = client
                .call(&alice(), &calls.next_item_id())
                .dry_run()
                .await?;
            assert_eq!(next_item.return_value(), 2);

            Ok(())
        }

        /// Burning clears ownership but item IDs continue to advance when minting again.
        #[ink_e2e::test(runtime)]
        async fn burn_and_reissue(mut client: Client) -> E2EResult<()> {
            let admin = alice().into_account_id();
            let create_args = encode_create_args(admin, default_collection_config(None));
            let mut constructor = NftsRuntimeCallerRef::new(0, create_args);

            let contract = client
                .instantiate("nfts_call_runtime", &alice(), &mut constructor)
                .value(1_000_000_000_000u128)
                .submit()
                .await?;

            let mut calls = contract.call_builder::<NftsRuntimeCaller>();

            let mint_args = encode_mint_args(
                0,
                0,
                to_runtime_account_id(contract.account_id.clone()),
            );
            let mint0 = client
                .call(&alice(), &calls.mint(mint_args))
                .submit()
                .await?;
            assert_ok!(mint0);
            assert!(client.runtime().item_exists(&0u32, &0u32));

            let burn_args = encode_burn_args(0, 0);
            let burn0 = client
                .call(&alice(), &calls.burn(burn_args))
                .submit()
                .await?;
            assert_ok!(burn0);
            assert!(!client.runtime().item_exists(&0u32, &0u32));
            assert!(client.runtime().owner_of(&0u32, &0u32).is_none());

            let mint_args = encode_mint_args(
                0,
                1,
                to_runtime_account_id(contract.account_id.clone()),
            );
            let mint1 = client
                .call(&alice(), &calls.mint(mint_args))
                .submit()
                .await?;
            assert_ok!(mint1);

            assert!(!client.runtime().item_exists(&0u32, &0u32));
            assert!(client.runtime().item_exists(&0u32, &1u32));

            let details = client
                .runtime()
                .collection_details(&0u32)
                .expect("collection should exist");
            assert_eq!(details.items, 1);

            let next_item = client
                .call(&alice(), &calls.next_item_id())
                .dry_run()
                .await?;
            assert_eq!(next_item.return_value(), 2);

            Ok(())
        }

        /// Transfer should fail for a non-existent item, with pallet error observed via
        /// runtime API.
        #[ink_e2e::test(runtime)]
        async fn transfer_nonexistent_item_fails(mut client: Client) -> E2EResult<()> {
            let admin = alice().into_account_id();
            let create_args = encode_create_args(admin, default_collection_config(None));
            let mut constructor = NftsRuntimeCallerRef::new(0, create_args);

            let contract = client
                .instantiate("nfts_call_runtime", &alice(), &mut constructor)
                .value(1_000_000_000_000u128)
                .submit()
                .await?;

            let _ = client.runtime().map_account(&bob());
            let bob_account = bob().into_account_id();

            let mut calls = contract.call_builder::<NftsRuntimeCaller>();

            // Verify the contract call fails.
            // Note: The specific pallet error (UnknownItem) is NOT propagated through
            // the XCM error path. The contract only sees a generic revert.
            let transfer_args = encode_transfer_args(0, 42, bob_account);
            let transfer_res = client
                .call(&alice(), &calls.transfer(transfer_args))
                .submit()
                .await?;
            assert!(transfer_res.dry_run.did_revert());

            // Verify the pallet error directly via runtime API.e the specific error.
            let runtime_err = client
                .runtime()
                .transfer(&0u32, 42u32, &bob())
                .expect_err("transfer of non-existent item should fail");
            assert!(format!("{:?}", runtime_err).contains("UnknownItem"));

            Ok(())
        }

        #[derive(Encode)]
        enum MultiAddress {
            Id(AccountId32),
        }

        fn unlookup(account: AccountId32) -> MultiAddress {
            MultiAddress::Id(account)
        }

        fn encode_create_args(
            admin: AccountId32,
            config: RuntimeCollectionConfig,
        ) -> Vec<u8> {
            (unlookup(admin), config).encode()
        }

        fn encode_mint_args(
            collection: CollectionId,
            item: ItemId,
            mint_to: AccountId32,
        ) -> Vec<u8> {
            (
                collection,
                item,
                unlookup(mint_to),
                Option::<RuntimeMintWitness>::None,
            )
                .encode()
        }

        fn encode_transfer_args(
            collection: CollectionId,
            item: ItemId,
            dest: AccountId32,
        ) -> Vec<u8> {
            (collection, item, unlookup(dest)).encode()
        }

        fn encode_burn_args(collection: CollectionId, item: ItemId) -> Vec<u8> {
            (collection, item).encode()
        }

        fn default_collection_config(max_supply: Option<u32>) -> RuntimeCollectionConfig {
            RuntimeCollectionConfig {
                settings: CollectionSettings::all_enabled(),
                max_supply,
                mint_settings: RuntimeMintSettings {
                    mint_type: MintType::Public,
                    ..Default::default()
                },
            }
        }

        fn to_runtime_account_id(id: AccountId) -> AccountId32 {
            AccountId32::from(*AsRef::<[u8; 32]>::as_ref(&id))
        }
    }
}
