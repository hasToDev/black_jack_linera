use std::str::FromStr;
use async_graphql::{Request, Response, scalar};
use async_graphql_derive::{SimpleObject};
use linera_sdk::base::{Amount, ChainId, ContractAbi, Owner, ServiceAbi, Timestamp};
use linera_sdk::graphql::GraphQLMutationRoot;
use serde::{Deserialize, Serialize};

pub struct BlackJackAbi;

impl ContractAbi for BlackJackAbi {
    type Operation = ();
    type Response = ();
}

impl ServiceAbi for BlackJackAbi {
    type Query = ();
    type QueryResponse = ();
}
