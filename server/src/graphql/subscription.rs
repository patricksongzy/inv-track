use std::pin::Pin;

use juniper::futures::{self, StreamExt};
use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::error::AppError;
use crate::graphql::{Clients, Context};
use crate::model::item::Item;
use crate::model::location::Location;
use crate::model::modification::Modification;
use crate::model::transaction::Transaction;

/// The root subscription.
pub(crate) struct RootSubscription;

/// A stream of modification results for a given object.
pub(crate) type ModificationStream<T> =
    Pin<Box<dyn futures::Stream<Item = Result<Modification<T>, AppError>> + Send + Sync>>;

/// Returns a subscription stream for a given type and channel name.
async fn subscription_stream<T: Serialize + DeserializeOwned>(
    clients: &Clients,
    channel_name: &str,
) -> ModificationStream<T> {
    let redis_conn = clients.redis.get_async_connection().await.unwrap();
    let mut pubsub = redis_conn.into_pubsub();
    pubsub
        .subscribe(channel_name)
        .await
        .expect("unable to subscribe to channel");
    let stream = pubsub.into_on_message().map(|message| {
        let payload: String = message.get_payload()?;
        serde_json::from_str(&payload).map_err(AppError::from)
    });

    Box::pin(stream)
}

/// The root subscription for the inventory tracking system.
#[graphql_subscription(context = Context)]
impl RootSubscription {
    /// The subscription to modifications of items.
    async fn item_subscription(context: &Context) -> ModificationStream<Item> {
        subscription_stream(&context.clients, "items").await
    }

    /// The subscription to modifications of transactions.
    async fn transaction_subscription(context: &Context) -> ModificationStream<Transaction> {
        subscription_stream(&context.clients, "transactions").await
    }

    /// The subscription to modifications of locations.
    async fn location_subscription(context: &Context) -> ModificationStream<Location> {
        subscription_stream(&context.clients, "locations").await
    }
}
