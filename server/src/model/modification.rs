use juniper::FieldError;
use redis::{AsyncCommands, RedisError};
use serde::{Deserialize, Serialize};

use crate::graphql::Context;
use crate::model::{item, location, transaction};

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
    result: &Result<T, FieldError>,
) {
    if let Ok(created) = &result {
        let modification = Modification {
            modification,
            data: created,
        };

        if let Ok(mut redis_conn) = context.redis.get_async_connection().await {
            let _: Result<(), RedisError> = redis_conn
                .publish(channel_name, serde_json::to_string(&modification).unwrap())
                .await;
        }
    }
}

/// A modification on an item.
#[graphql_object(name = "ItemModification", context = Context)]
impl Modification<item::Item> {
    /// The item modified.
    fn item(&self) -> &item::Item {
        &self.data
    }

    /// The modification type.
    fn modification(&self) -> &ModificationType {
        &self.modification
    }
}

/// A modification on a transaction.
#[graphql_object(name = "TransactionModification", context = Context)]
impl Modification<transaction::Transaction> {
    /// The transaction modified.
    fn transaction(&self) -> &transaction::Transaction {
        &self.data
    }

    /// The modification type.
    fn modification(&self) -> &ModificationType {
        &self.modification
    }
}

/// A modification on a location.
#[graphql_object(name = "LocationModification", context = Context)]
impl Modification<location::Location> {
    /// The location modified.
    fn location(&self) -> &location::Location {
        &self.data
    }

    /// The modification type.
    fn modification(&self) -> &ModificationType {
        &self.modification
    }
}
