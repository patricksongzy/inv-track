use async_graphql::{Context, Result};

use crate::graphql::AppContext;
use crate::model::{item, location, transaction};

/// The item query.
#[derive(Default)]
struct ItemQuery;
/// The location query.
#[derive(Default)]
struct LocationQuery;
/// The transaction query.
#[derive(Default)]
struct TransactionQuery;

/// The root query.
#[derive(async_graphql::MergedObject, Default)]
pub(crate) struct RootQuery(ItemQuery, LocationQuery, TransactionQuery);

/// The item query for the inventory tracking system.
#[async_graphql::Object]
impl ItemQuery {
    /// The query to retrieve all items.
    async fn items(&self, context: &Context<'_>) -> Result<Vec<item::Item>> {
        item::get_items(context.data_unchecked::<AppContext>()).await
    }

    /// The query to retrieve a single item by id.
    async fn item(&self, context: &Context<'_>, id: item::ItemId) -> Result<item::Item> {
        item::get_item(context.data_unchecked::<AppContext>(), id).await
    }
}

/// The location query for the inventory tracking system.
#[async_graphql::Object]
impl LocationQuery {
    /// The query to retrieve all locations.
    async fn locations(&self, context: &Context<'_>) -> Result<Vec<location::Location>> {
        location::get_locations(context.data_unchecked::<AppContext>()).await
    }

    /// The query to retrieve a single location by id.
    async fn location(
        &self,
        context: &Context<'_>,
        id: location::LocationId,
    ) -> Result<location::Location> {
        location::get_location(context.data_unchecked::<AppContext>(), id).await
    }
}

/// The transaction query for the inventory tracking system.
#[async_graphql::Object]
impl TransactionQuery {
    /// The query to retrieve all transactions.
    async fn transactions(&self, context: &Context<'_>) -> Result<Vec<transaction::Transaction>> {
        transaction::get_transactions(context.data_unchecked::<AppContext>()).await
    }

    /// The query to retrieve a single transaction by id.
    async fn transaction(
        &self,
        context: &Context<'_>,
        id: transaction::TransactionId,
    ) -> Result<transaction::Transaction> {
        transaction::get_transaction(context.data_unchecked::<AppContext>(), id).await
    }
}
