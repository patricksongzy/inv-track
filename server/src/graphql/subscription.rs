use std::pin::Pin;

use async_graphql::{Context, Error, Result};
use serde::de::DeserializeOwned;
use serde::Serialize;
use tokio_stream::StreamExt;

use crate::graphql::{AppContext, Clients};
use crate::model::item::Item;
use crate::model::location::Location;
use crate::model::modification::Modification;
use crate::model::transaction::Transaction;

/// The item subscription.
#[derive(Default)]
struct ItemSubscription;
/// The location subscription.
#[derive(Default)]
struct LocationSubscription;
/// The transaction subscription.
#[derive(Default)]
struct TransactionSubscription;

/// The root subscription.
#[derive(async_graphql::MergedSubscription, Default)]
pub(crate) struct RootSubscription(ItemSubscription, LocationSubscription, TransactionSubscription);

/// A stream of modification results for a given object.
pub(crate) type ModificationStream<T> =
    Pin<Box<dyn futures::Stream<Item = Result<Modification<T>>> + Send + Sync>>;

/// Returns a subscription stream for a given type and channel name.
async fn subscription_stream<T: Serialize + DeserializeOwned + async_graphql::OutputType>(
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
        serde_json::from_str(&payload).map_err(Error::from)
    });

    Box::pin(stream)
}

/// The item subscription for the inventory tracking system.
#[async_graphql::Subscription]
impl ItemSubscription {
    /// The subscription to modifications of items.
    async fn item_subscription(&self, context: &Context<'_>) -> ModificationStream<Item> {
        subscription_stream(&context.data_unchecked::<AppContext>().clients, "items").await
    }
}

/// The location subscription for the inventory tracking system.
#[async_graphql::Subscription]
impl LocationSubscription {
    /// The subscription to modifications of locations.
    async fn location_subscription(&self, context: &Context<'_>) -> ModificationStream<Location> {
        subscription_stream(&context.data_unchecked::<AppContext>().clients, "locations").await
    }
}

/// The Transaction subscription for the inventory tracking system.
#[async_graphql::Subscription]
impl TransactionSubscription {
    /// The subscription to modifications of transactions.
    async fn transaction_subscription(&self, context: &Context<'_>) -> ModificationStream<Transaction> {
        subscription_stream(&context.data_unchecked::<AppContext>().clients, "transactions").await
    }
}
