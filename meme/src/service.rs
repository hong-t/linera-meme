// Copyright (c) Zefchain Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

#![cfg_attr(target_arch = "wasm32", no_main)]

mod state;

use std::{str::FromStr, sync::Arc};

use abi::meme::{MemeAbi, MemeOperation};
use async_graphql::{EmptySubscription, Object, Request, Response, Schema};
use linera_sdk::{
    linera_base_types::{Account, Amount, WithServiceAbi},
    views::View,
    Service, ServiceRuntime,
};

use self::state::MemeState;

pub struct MemeService {
    state: Arc<MemeState>,
    runtime: Arc<ServiceRuntime<Self>>,
}

linera_sdk::service!(MemeService);

impl WithServiceAbi for MemeService {
    type Abi = MemeAbi;
}

impl Service for MemeService {
    type Parameters = ();

    async fn new(runtime: ServiceRuntime<Self>) -> Self {
        let state = MemeState::load(runtime.root_view_storage_context())
            .await
            .expect("Failed to load state");
        MemeService {
            state: Arc::new(state),
            runtime: Arc::new(runtime),
        }
    }

    async fn handle_query(&self, request: Request) -> Response {
        let schema = Schema::build(
            QueryRoot {
                state: self.state.clone(),
            },
            MutationRoot {
                runtime: self.runtime.clone(),
            },
            EmptySubscription,
        )
        .finish();
        schema.execute(request).await
    }
}

struct MutationRoot {
    runtime: Arc<ServiceRuntime<MemeService>>,
}

#[Object]
impl MutationRoot {
    async fn transfer(&self, to: Account, amount: Amount) -> [u8; 0] {
        self.runtime
            .schedule_operation(&MemeOperation::Transfer { to, amount });
        []
    }
}

struct QueryRoot {
    state: Arc<MemeState>,
}

#[Object]
impl QueryRoot {
    async fn total_supply(&self) -> Amount {
        self.state.meme.get().as_ref().unwrap().total_supply
    }

    // async fn balance_of(&self, owner: Account) -> Amount {
    async fn balance_of(&self, owner: String) -> Amount {
        self.state
            .balance_of(Account::from_str(&owner).unwrap())
            .await
    }

    // async fn allowance_of(&self, owner: Account, spender: Account) -> Amount {
    async fn allowance_of(&self, owner: String, spender: String) -> Amount {
        self.state
            .allowance_of(
                Account::from_str(&owner).unwrap(),
                Account::from_str(&spender).unwrap(),
            )
            .await
    }

    async fn initial_owner_balance(&self) -> Amount {
        self.state.initial_owner_balance().await
    }

    async fn liquidity_pool_initialized(&self) -> bool {
        self.state.liquidity_pool_initialized().await
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use abi::{
        meme::{InstantiationArgument, Meme, Metadata},
        store_type::StoreType,
    };
    use async_graphql::{Request, Response, Value};
    use futures::FutureExt as _;
    use linera_sdk::{
        linera_base_types::{Account, AccountOwner, Amount, ApplicationId, ChainId, Owner},
        util::BlockingWait,
        views::View,
        Service, ServiceRuntime,
    };
    use serde_json::json;
    use std::str::FromStr;

    use super::{MemeService, MemeState};

    #[tokio::test(flavor = "multi_thread")]
    async fn query() {
        let runtime = Arc::new(ServiceRuntime::<MemeService>::new());
        let mut state = MemeState::load(runtime.root_view_storage_context())
            .blocking_wait()
            .expect("Failed to read from mock key value store");

        let application_id_str = "b94e486abcfc016e937dad4297523060095f405530c95d498d981a94141589f167693295a14c3b48460ad6f75d67d2414428227550eb8cee8ecaa37e8646518300aee928d4bf3880353b4a3cd9b6f88e6cc6e5ed050860abae439e7782e9b2dfe8020000000000000000000000";
        let application_id = ApplicationId::from_str(application_id_str).unwrap();
        let instantiation_argument = InstantiationArgument {
            meme: Meme {
                name: "Test Token".to_string(),
                ticker: "LTT".to_string(),
                decimals: 6,
                initial_supply: Amount::from_tokens(21000000),
                total_supply: Amount::from_tokens(21000000),
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
            swap_application_id: Some(application_id),
        };

        let chain_id =
            ChainId::from_str("899dd894c41297e9dd1221fa02845efc81ed8abd9a0b7d203ad514b3aa6b2d46")
                .unwrap();
        let owner = Account {
            chain_id,
            owner: Some(AccountOwner::User(
                Owner::from_str("02e900512d2fca22897f80a2f6932ff454f2752ef7afad18729dd25e5b5b6e00")
                    .unwrap(),
            )),
        };
        let application_id_str = "b94e486abcfc016e937dad4297523060095f405530c95d498d981a94141589f167693295a14c3b48460ad6f75d67d2414428227550eb8cee8ecaa37e8646518300aee928d4bf3880353b4a3cd9b6f88e6cc6e5ed050860abae439e7782e9b2dfe8020000000000000000000000";
        let application_id = ApplicationId::from_str(application_id_str).unwrap();
        let application = Account {
            chain_id,
            owner: Some(AccountOwner::Application(application_id)),
        };
        state
            .instantiate(owner, application, instantiation_argument.clone())
            .await
            .expect("Failed inistantiate");

        let service = MemeService {
            state: Arc::new(state),
            runtime,
        };
        let request = Request::new("{ totalSupply }");

        let response = service
            .handle_query(request)
            .now_or_never()
            .expect("Query should not await anything");

        let expected = Response::new(
            Value::from_json(json!({"totalSupply" : instantiation_argument.meme.total_supply}))
                .unwrap(),
        );

        assert_eq!(response, expected)
    }
}
