// Copyright (c) Zefchain Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

//! Integration tests for the Swap application.

#![cfg(not(target_arch = "wasm32"))]

use abi::{
    meme::{
        InstantiationArgument as MemeInstantiationArgument, Liquidity, Meme, MemeAbi,
        MemeOperation, MemeParameters, Metadata,
    },
    store_type::StoreType,
    swap::router::{
        InstantiationArgument as SwapInstantiationArgument, Pool, SwapAbi, SwapParameters,
    },
};
use linera_sdk::{
    linera_base_types::{
        Account, AccountOwner, Amount, ApplicationId, ChainDescription, ChainId, MessageId,
        ModuleId, Owner,
    },
    test::{ActiveChain, Medium, MessageAction, QueryOutcome, Recipient, TestValidator},
};
use std::str::FromStr;

#[derive(Clone)]
struct TestSuite {
    pub validator: TestValidator,

    pub admin_chain: ActiveChain,
    pub meme_chain: ActiveChain,
    pub user_chain: ActiveChain,
    pub swap_chain: ActiveChain,

    pub swap_application_id: Option<ApplicationId<SwapAbi>>,
    pub meme_application_id: Option<ApplicationId<MemeAbi>>,

    pub swap_bytecode_id: ModuleId<SwapAbi, SwapParameters, SwapInstantiationArgument>,
    pub meme_bytecode_id: ModuleId<MemeAbi, MemeParameters, MemeInstantiationArgument>,

    pub initial_supply: Amount,
    pub initial_liquidity: Amount,
}

impl TestSuite {
    async fn new() -> Self {
        let (validator, swap_bytecode_id) = TestValidator::with_current_module::<
            SwapAbi,
            SwapParameters,
            SwapInstantiationArgument,
        >()
        .await;

        let admin_chain = validator.get_chain(&ChainId::root(0));
        let meme_chain = validator.new_chain().await;
        let user_chain = validator.new_chain().await;
        let swap_chain = validator.new_chain().await;

        let meme_bytecode_id = swap_chain.publish_bytecode_files_in("../meme").await;

        TestSuite {
            validator,

            admin_chain,
            meme_chain,
            user_chain,
            swap_chain,

            swap_application_id: None,
            meme_application_id: None,

            swap_bytecode_id,
            meme_bytecode_id,

            initial_supply: Amount::from_tokens(21000000),
            initial_liquidity: Amount::from_tokens(11000000),
        }
    }

    fn chain_account(&self, chain: ActiveChain) -> Account {
        Account {
            chain_id: chain.id(),
            owner: None,
        }
    }

    fn chain_owner_account(&self, chain: &ActiveChain) -> Account {
        Account {
            chain_id: chain.id(),
            owner: Some(AccountOwner::User(Owner::from(chain.public_key()))),
        }
    }

    fn application_account(&self, application_id: ApplicationId) -> Account {
        Account {
            chain_id: application_id.creation.chain_id,
            owner: Some(AccountOwner::Application(application_id.forget_abi())),
        }
    }

    async fn fund_chain(&self, chain: &ActiveChain, amount: Amount) {
        let certificate = self
            .admin_chain
            .add_block(|block| {
                block.with_native_token_transfer(
                    None,
                    Recipient::Account(self.chain_account(chain.clone())),
                    amount,
                );
            })
            .await;
        chain
            .add_block(move |block| {
                block.with_messages_from_by_medium(
                    &certificate,
                    &Medium::Direct,
                    MessageAction::Accept,
                );
            })
            .await;
        chain.handle_received_messages().await;
    }

    async fn create_swap_application(&mut self) {
        let liquidity_rfq_bytecode_id = self
            .swap_chain
            .publish_bytecode_files_in("../liquidity-rfq")
            .await;
        let pool_bytecode_id = self.swap_chain.publish_bytecode_files_in("../pool").await;

        self.swap_application_id = Some(
            self.swap_chain
                .create_application::<SwapAbi, SwapParameters, SwapInstantiationArgument>(
                    self.swap_bytecode_id,
                    SwapParameters {},
                    SwapInstantiationArgument {
                        liquidity_rfq_bytecode_id,
                        pool_bytecode_id,
                    },
                    vec![],
                )
                .await,
        )
    }

    async fn create_meme_application(&mut self) {
        let instantiation_argument = MemeInstantiationArgument {
            meme: Meme {
                name: "Test Token".to_string(),
                ticker: "LTT".to_string(),
                decimals: 6,
                initial_supply: self.initial_supply,
                total_supply: self.initial_supply,
                metadata: Metadata {
                    logo_store_type: StoreType::S3,
                    logo: "Test Logo".to_string(),
                    description: "Test token description".to_string(),
                    twitter: None,
                    telegram: None,
                    discord: None,
                    website: None,
                    github: None,
                },
            },
            blob_gateway_application_id: None,
            ams_application_id: None,
            proxy_application_id: None,
            swap_application_id: Some(self.swap_application_id.unwrap().forget_abi()),
        };
        let parameters = MemeParameters {
            creator: self.chain_owner_account(&self.meme_chain),
            initial_liquidity: Some(Liquidity {
                fungible_amount: self.initial_liquidity,
                native_amount: Amount::from_tokens(10),
            }),
            virtual_initial_liquidity: true,
        };

        self.meme_application_id = Some(
            self.meme_chain
                .create_application(
                    self.meme_bytecode_id,
                    parameters.clone(),
                    instantiation_argument.clone(),
                    vec![],
                )
                .await,
        )
    }

    async fn initialize_liquidity(&self, chain: &ActiveChain) {
        chain
            .add_block(|block| {
                block.with_operation(
                    self.meme_application_id.unwrap(),
                    MemeOperation::InitializeLiquidity,
                );
            })
            .await;
        self.meme_chain.handle_received_messages().await;
    }
}

/// Test setting a swap and testing its coherency across microchains.
///
/// Creates the application on a `chain`, initializing it with a 42 then adds 15 and obtains 57.
/// which is then checked.
#[tokio::test(flavor = "multi_thread")]
async fn virtual_liquidity_test() {
    let _ = env_logger::builder().is_test(true).try_init();

    let _ = env_logger::builder().is_test(true).try_init();

    let mut suite = TestSuite::new().await;
    let meme_chain = suite.meme_chain.clone();
    let user_chain = suite.user_chain.clone();
    let swap_chain = suite.swap_chain.clone();

    let swap_key_pair = swap_chain.key_pair();

    suite.fund_chain(&meme_chain, Amount::ONE).await;

    suite.create_swap_application().await;
    suite.create_meme_application().await;

    meme_chain.handle_received_messages().await;
    swap_chain.handle_received_messages().await;

    // Here we initialize liquidity pool
    meme_chain
        .register_application(suite.swap_application_id.unwrap().forget_abi())
        .await;
    swap_chain
        .register_application(suite.meme_application_id.unwrap().forget_abi())
        .await;

    let query = format!(
        "query {{ allowanceOf(owner: \"{}\", spender: \"{}\") }}",
        suite.application_account(suite.meme_application_id.unwrap().forget_abi()),
        suite.application_account(suite.swap_application_id.unwrap().forget_abi()),
    );
    let QueryOutcome { response, .. } = meme_chain
        .graphql_query(suite.meme_application_id.unwrap(), query)
        .await;
    assert_eq!(
        Amount::from_str(response["allowanceOf"].as_str().unwrap()).unwrap(),
        suite.initial_liquidity,
    );

    suite.initialize_liquidity(&meme_chain).await;

    meme_chain.handle_received_messages().await;
    swap_chain.handle_received_messages().await;

    let QueryOutcome { response, .. } = swap_chain
        .graphql_query(
            suite.swap_application_id.unwrap(),
            "query { poolChainCreationMessages }",
        )
        .await;
    assert_eq!(
        response["poolChainCreationMessages"]
            .as_array()
            .unwrap()
            .len(),
        1,
    );

    let message_id = MessageId::from_str(
        response["poolChainCreationMessages"].as_array().unwrap()[0]
            .as_str()
            .unwrap(),
    )
    .unwrap();
    let description = ChainDescription::Child(message_id);
    let pool_chain = ActiveChain::new(swap_key_pair.copy(), description, suite.clone().validator);
    suite.validator.add_chain(pool_chain.clone());

    pool_chain.handle_received_messages().await;
    swap_chain.handle_received_messages().await;
    meme_chain.handle_received_messages().await;

    let QueryOutcome { response, .. } = swap_chain
        .graphql_query(
            suite.swap_application_id.unwrap(),
            "query { pools {
                poolId
                token0
                token1
                poolApplication
            }}",
        )
        .await;
    assert_eq!(response["pools"].as_array().unwrap().len(), 1,);

    let pool: Pool =
        serde_json::from_value(response["pools"].as_array().unwrap()[0].clone()).unwrap();

    let query = format!(
        "query {{ allowanceOf(owner: \"{}\", spender: \"{}\") }}",
        suite.application_account(suite.meme_application_id.unwrap().forget_abi()),
        suite.application_account(suite.swap_application_id.unwrap().forget_abi()),
    );
    let QueryOutcome { response, .. } = meme_chain
        .graphql_query(suite.meme_application_id.unwrap(), query)
        .await;
    assert_eq!(
        Amount::from_str(response["allowanceOf"].as_str().unwrap()).unwrap(),
        Amount::ZERO,
    );

    let query = format!("query {{ balanceOf(owner: \"{}\")}}", pool.pool_application,);
    let QueryOutcome { response, .. } = meme_chain
        .graphql_query(suite.meme_application_id.unwrap(), query)
        .await;
    assert_eq!(
        Amount::from_str(response["balanceOf"].as_str().unwrap()).unwrap(),
        suite.initial_liquidity,
    );

    // TODO: check native balance of pool chain
}
