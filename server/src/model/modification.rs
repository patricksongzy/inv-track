use redis::{AsyncCommands, RedisError};
use serde::{Deserialize, Serialize};

use crate::graphql::AppContext;
use crate::model::item::Item;
use crate::model::location::Location;
use crate::model::transaction::Transaction;

/// The type of modification.
#[derive(Copy, Clone, PartialEq, Eq, Serialize, Deserialize, async_graphql::Enum)]
pub(crate) enum ModificationType {
    Create,
    Update,
    Delete,
}

/// The modification to broadcast to subscribers.
#[derive(async_graphql::SimpleObject, Serialize, Deserialize)]
#[graphql(concrete(name = "ItemModification", params(Item)))]
#[graphql(concrete(name = "LocationModification", params(Location)))]
#[graphql(concrete(name = "TransactionModification", params(Transaction)))]
pub(crate) struct Modification<T: Serialize + async_graphql::OutputType> {
    pub(crate) modification: ModificationType,
    pub(crate) data: T,
}

/// Broadcasts a modification to subscribers to a given channel, containing the modification type and data.
pub(crate) async fn broadcast<T: Serialize + async_graphql::OutputType>(
    context: &AppContext,
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
