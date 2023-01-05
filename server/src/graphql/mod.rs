mod mutation;
mod query;
mod subscription;

use std::sync::Arc;

use async_graphql::{Schema, SchemaBuilder};
use sqlx::{Pool, Postgres};

use crate::graphql::mutation::RootMutation;
use crate::graphql::query::RootQuery;
use crate::graphql::subscription::RootSubscription;

/// The clients of the application.
#[derive(Clone)]
pub(crate) struct Clients {
    pub(crate) postgres: Arc<Pool<Postgres>>,
    pub(crate) redis: Arc<redis::Client>,
}

/// The context of the application.
#[derive(Clone)]
pub(crate) struct AppContext {
    pub(crate) clients: Clients,
    pub(crate) loaders: Arc<anymap2::Map<dyn anymap2::any::Any + Send + Sync>>,
}

pub(crate) type AppSchema = Schema<RootQuery, RootMutation, RootSubscription>;

/// Returns a created schema for the application.
pub(crate) fn schema_builder() -> SchemaBuilder<RootQuery, RootMutation, RootSubscription> {
    Schema::build(
        RootQuery::default(),
        RootMutation::default(),
        RootSubscription::default(),
    )
}
