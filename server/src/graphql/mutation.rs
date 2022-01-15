use async_graphql::{Context, Result};

use crate::graphql::AppContext;
use crate::model::{item, location, transaction};

/// The item mutation.
#[derive(Default)]
struct ItemMutation;
/// The location mutation.
#[derive(Default)]
struct LocationMutation;
/// The transaction mutation.
#[derive(Default)]
struct TransactionMutation;

/// The root mutation.
#[derive(async_graphql::MergedObject, Default)]
pub(crate) struct RootMutation(ItemMutation, LocationMutation, TransactionMutation);

/// The item mutation for the inventory tracking system.
#[async_graphql::Object]
impl ItemMutation {
    /// The mutation to create an item.
    async fn create_item(
        &self,
        context: &Context<'_>,
        item: item::InsertableItem,
    ) -> Result<item::Item> {
        item::create_item(context.data_unchecked::<AppContext>(), item).await
    }

    /// The mutation to update an item with the given id.
    async fn update_item(
        &self,
        context: &Context<'_>,
        id: item::ItemId,
        item: item::InsertableItem,
    ) -> Result<item::Item> {
        item::update_item(context.data_unchecked::<AppContext>(), id, item).await
    }

    /// The mutation to delete an item with the given id.
    async fn delete_item(&self, context: &Context<'_>, id: item::ItemId) -> Result<item::Item> {
        item::delete_item(context.data_unchecked::<AppContext>(), id).await
    }
}

/// The location mutation for the inventory tracking system.
#[async_graphql::Object]
impl LocationMutation {
    /// The mutation to create a location.
    async fn create_location(
        &self,
        context: &Context<'_>,
        location: location::InsertableLocation,
    ) -> Result<location::Location> {
        location::create_location(context.data_unchecked::<AppContext>(), location).await
    }

    /// The mutation to update a location with the given id.
    async fn update_location(
        &self,
        context: &Context<'_>,
        id: location::LocationId,
        location: location::InsertableLocation,
    ) -> Result<location::Location> {
        location::update_location(context.data_unchecked::<AppContext>(), id, location).await
    }

    /// The mutation to delete a location with the given id.
    async fn delete_location(
        &self,
        context: &Context<'_>,
        id: location::LocationId,
    ) -> Result<location::Location> {
        location::delete_location(context.data_unchecked::<AppContext>(), id).await
    }
}

/// The transaction mutation for the inventory tracking system.
#[async_graphql::Object]
impl TransactionMutation {
    /// The mutation to create a transaction.
    async fn create_transaction(
        &self,
        context: &Context<'_>,
        transaction: transaction::InsertableTransaction,
    ) -> Result<transaction::Transaction> {
        transaction::create_transaction(context.data_unchecked::<AppContext>(), transaction).await
    }

    /// The mutation to update a transaction with the given id.
    async fn update_transaction(
        &self,
        context: &Context<'_>,
        id: transaction::TransactionId,
        transaction: transaction::InsertableTransaction,
    ) -> Result<transaction::Transaction> {
        transaction::update_transaction(context.data_unchecked::<AppContext>(), id, transaction).await
    }

    /// The mutation to delete a transaction with the given id.
    async fn delete_transaction(
        &self,
        context: &Context<'_>,
        id: transaction::TransactionId,
    ) -> Result<transaction::Transaction> {
        transaction::delete_transaction(context.data_unchecked::<AppContext>(), id).await
    }
}
