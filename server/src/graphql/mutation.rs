use juniper::FieldError;

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
    ) -> Result<item::Item, FieldError> {
        item::create_item(context, item).await
    }

    /// The mutation to update an item with the given id.
    async fn update_item(
        context: &Context,
        id: i32,
        item: item::InsertableItem,
    ) -> Result<item::Item, FieldError> {
        item::update_item(context, id, item).await
    }

    /// The mutation to delete an item with the given id.
    async fn delete_item(context: &Context, id: i32) -> Result<item::Item, FieldError> {
        item::delete_item(context, id).await
    }

    /// The mutation to create a transaction.
    async fn create_transaction(
        context: &Context,
        transaction: transaction::InsertableTransaction,
    ) -> Result<transaction::Transaction, FieldError> {
        transaction::create_transaction(context, transaction).await
    }

    /// The mutation to update a transaction with the given id.
    async fn update_transaction(
        context: &Context,
        id: i32,
        transaction: transaction::InsertableTransaction,
    ) -> Result<transaction::Transaction, FieldError> {
        transaction::update_transaction(context, id, transaction).await
    }

    /// The mutation to delete a transaction with the given id.
    async fn delete_transaction(
        context: &Context,
        id: i32,
    ) -> Result<transaction::Transaction, FieldError> {
        transaction::delete_transaction(context, id).await
    }

    /// The mutation to create a location.
    async fn create_location(
        context: &Context,
        location: location::InsertableLocation,
    ) -> Result<location::Location, FieldError> {
        location::create_location(context, location).await
    }

    /// The mutation to update a location with the given id.
    async fn update_location(
        context: &Context,
        id: i32,
        location: location::InsertableLocation,
    ) -> Result<location::Location, FieldError> {
        location::update_location(context, id, location).await
    }

    /// The mutation to delete a location with the given id.
    async fn delete_location(context: &Context, id: i32) -> Result<location::Location, FieldError> {
        location::delete_location(context, id).await
    }
}
