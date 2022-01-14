use std::pin::Pin;

use juniper::futures::{self, StreamExt};
use juniper::FieldError;
use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::graphql::Context;
use crate::model::modification::Modification;
use crate::model::{item, location, transaction};

/// The root subscription.
pub(crate) struct RootSubscription;

/// A stream of modification results for a given object.
pub(crate) type ModificationStream<T> =
    Pin<Box<dyn futures::Stream<Item = Result<Modification<T>, FieldError>> + Send>>;

/// Returns a subscription stream for a given type and channel name.
async fn subscription_stream<T: Serialize + DeserializeOwned>(
    context: &Context,
    channel_name: &str,
) -> ModificationStream<T> {
    let redis_conn = context.redis.get_async_connection().await.unwrap(); // TODO FIX UNWRAP
    let mut pubsub = redis_conn.into_pubsub();
    pubsub
        .subscribe(channel_name)
        .await
        .expect("unable to subscribe to channel");
    let stream = pubsub.into_on_message().map(|message| {
        let payload: String = message.get_payload()?;
        serde_json::from_str(&payload).map_err(FieldError::from)
    });

    Box::pin(stream)
}

/// The root subscription for the inventory tracking system.
#[graphql_subscription(context = Context)]
impl RootSubscription {
    /// The subscription to modifications of items.
    async fn item_subscription(context: &Context) -> ModificationStream<item::Item> {
        subscription_stream(context, "items").await
    }

    /// The subscription to modifications of transactions.
    async fn transaction_subscription(
        context: &Context,
    ) -> ModificationStream<transaction::Transaction> {
        subscription_stream(context, "transactions").await
    }

    /// The subscription to modifications of locations.
    async fn location_subscription(context: &Context) -> ModificationStream<location::Location> {
        subscription_stream(context, "locations").await
    }
}
