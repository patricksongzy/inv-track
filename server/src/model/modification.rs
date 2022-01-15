use redis::{AsyncCommands, RedisError};
use serde::{Deserialize, Serialize};

use crate::graphql::Context;
use crate::model::item::Item;
use crate::model::location::Location;
use crate::model::transaction::Transaction;

/// The type of modification.
#[derive(Serialize, Deserialize, GraphQLEnum)]
pub(crate) enum ModificationType {
    Create,
    Update,
    Delete,
}

/// The modification to broadcast to subscribers.
#[derive(Serialize, Deserialize)]
pub(crate) struct Modification<T: Serialize> {
    pub(crate) modification: ModificationType,
    pub(crate) data: T,
}

/// Broadcasts a modification to subscribers to a given channel, containing the modification type and data.
pub(crate) async fn broadcast<T: Serialize>(
    context: &Context,
    channel_name: &str,
    modification: ModificationType,
    created: &T,
) {
    let modification = Modification {
        modification,
        data: created,
    };

    if let Ok(mut redis_conn) = context.clients.redis.get_async_connection().await {
        let _: Result<(), RedisError> = redis_conn
            .publish(channel_name, serde_json::to_string(&modification).unwrap())
            .await;
    }
}

/// A modification on an item.
#[graphql_object(name = "ItemModification", context = Context)]
impl Modification<Item> {
    /// The item modified.
    fn item(&self) -> &Item {
        &self.data
    }

    /// The modification type.
    fn modification(&self) -> &ModificationType {
        &self.modification
    }
}

/// A modification on a transaction.
#[graphql_object(name = "TransactionModification", context = Context)]
impl Modification<Transaction> {
    /// The transaction modified.
    fn transaction(&self) -> &Transaction {
        &self.data
    }

    /// The modification type.
    fn modification(&self) -> &ModificationType {
        &self.modification
    }
}

/// A modification on a location.
#[graphql_object(name = "LocationModification", context = Context)]
impl Modification<Location> {
    /// The location modified.
    fn location(&self) -> &Location {
        &self.data
    }

    /// The modification type.
    fn modification(&self) -> &ModificationType {
        &self.modification
    }
}
