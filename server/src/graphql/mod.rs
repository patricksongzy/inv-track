mod mutation;
mod query;
mod subscription;

use std::sync::Arc;

use juniper::RootNode;
use sqlx::{Pool, Postgres};

use crate::graphql::mutation::RootMutation;
use crate::graphql::query::RootQuery;
use crate::graphql::subscription::RootSubscription;

/// The context of the application.
#[derive(Clone)]
pub(crate) struct Context {
    pub(crate) pool: Arc<Pool<Postgres>>,
    pub(crate) redis: Arc<redis::Client>,
}

/// Marker implementation for juniper contexts.
impl juniper::Context for Context {}

/// The juniper schema.
pub(crate) type Schema = RootNode<'static, RootQuery, RootMutation, RootSubscription>;

/// Returns a created juniper schema for the application.
pub(crate) fn schema() -> Schema {
    Schema::new(RootQuery {}, RootMutation {}, RootSubscription {})
}
