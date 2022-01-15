use crate::error::AppError;
use crate::graphql::Context;
use crate::model::{item, location, transaction};

/// The root mutation.
pub(crate) struct RootMutation;

/// The root mutation for the inventory tracking system.
#[graphql_object(context = Context)]
impl RootMutation {
    /// The mutation to create an item.
    async fn create_item(
        context: &Context,
        item: item::InsertableItem,
    ) -> Result<item::Item, AppError> {
        item::create_item(context, item).await
    }

    /// The mutation to update an item with the given id.
    async fn update_item(
        context: &Context,
        id: item::ItemId,
        item: item::InsertableItem,
    ) -> Result<item::Item, AppError> {
        item::update_item(context, id, item).await
    }

    /// The mutation to delete an item with the given id.
    async fn delete_item(context: &Context, id: item::ItemId) -> Result<item::Item, AppError> {
        item::delete_item(context, id).await
    }

    /// The mutation to create a transaction.
    async fn create_transaction(
        context: &Context,
        transaction: transaction::InsertableTransaction,
    ) -> Result<transaction::Transaction, AppError> {
        transaction::create_transaction(context, transaction).await
    }

    /// The mutation to update a transaction with the given id.
    async fn update_transaction(
        context: &Context,
        id: transaction::TransactionId,
        transaction: transaction::InsertableTransaction,
    ) -> Result<transaction::Transaction, AppError> {
        transaction::update_transaction(context, id, transaction).await
    }

    /// The mutation to delete a transaction with the given id.
    async fn delete_transaction(
        context: &Context,
        id: transaction::TransactionId,
    ) -> Result<transaction::Transaction, AppError> {
        transaction::delete_transaction(context, id).await
    }

    /// The mutation to create a location.
    async fn create_location(
        context: &Context,
        location: location::InsertableLocation,
    ) -> Result<location::Location, AppError> {
        location::create_location(context, location).await
    }

    /// The mutation to update a location with the given id.
    async fn update_location(
        context: &Context,
        id: location::LocationId,
        location: location::InsertableLocation,
    ) -> Result<location::Location, AppError> {
        location::update_location(context, id, location).await
    }

    /// The mutation to delete a location with the given id.
    async fn delete_location(
        context: &Context,
        id: location::LocationId,
    ) -> Result<location::Location, AppError> {
        location::delete_location(context, id).await
    }
}
