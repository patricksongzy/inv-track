use juniper::FieldError;

use crate::graphql::Context;
use crate::model::{item, location, transaction};

/// The root query.
pub(crate) struct RootQuery;

/// The root query for the inventory tracking system.
#[graphql_object(context = Context)]
impl RootQuery {
    /// The query to retrieve all items.
    async fn items(context: &Context) -> Result<Vec<item::Item>, FieldError> {
        item::get_items(context).await
    }

    /// The query to retrieve a single item by id.
    async fn item(context: &Context, id: i32) -> Result<item::Item, FieldError> {
        item::get_item(context, id).await
    }

    /// The query to retrieve all transactions.
    async fn transactions(context: &Context) -> Result<Vec<transaction::Transaction>, FieldError> {
        transaction::get_transactions(context).await
    }

    /// The query to retrieve a single transaction by id.
    async fn transaction(
        context: &Context,
        id: i32,
    ) -> Result<transaction::Transaction, FieldError> {
        transaction::get_transaction(context, id).await
    }

    /// The query to retrieve all locations.
    async fn locations(context: &Context) -> Result<Vec<location::Location>, FieldError> {
        location::get_locations(context).await
    }

    /// The query to retrieve a single location by id.
    async fn location(context: &Context, id: i32) -> Result<location::Location, FieldError> {
        location::get_location(context, id).await
    }
}
