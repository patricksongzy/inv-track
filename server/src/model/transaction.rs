use juniper::FieldError;
use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::graphql::Context;
use crate::model::modification::{self, ModificationType};
use crate::model::{item, location};

/// Transaction model returned by a query in the inventory tracking system.
#[derive(Debug, PartialEq, sqlx::FromRow, Serialize, Deserialize)]
pub(crate) struct Transaction {
    id: i32,
    item_id: i32,
    location_id: Option<i32>,
    quantity: i32,
    comment: Option<String>,
}

/// Transaction model to insert in the inventory tracking system.
#[derive(Debug, PartialEq, Validate, Deserialize, GraphQLInputObject)]
#[graphql(description = "A transaction to input to the inventory system.")]
pub(crate) struct InsertableTransaction {
    item_id: i32,
    location_id: Option<i32>,
    quantity: i32,
    comment: Option<String>,
}

/// Gets all transactions, returning the result, or a field error.
pub(crate) async fn get_transactions(context: &Context) -> Result<Vec<Transaction>, FieldError> {
    sqlx::query_as::<_, Transaction>(
        r#"
        select id, item_id, location_id, quantity, comment from transactions
    "#,
    )
    .fetch_all(&*context.pool)
    .await
    .map_err(FieldError::from)
}

/// Gets an transaction, given an id, returning the result, or a field error.
pub(crate) async fn get_transaction(context: &Context, id: i32) -> Result<Transaction, FieldError> {
    sqlx::query_as::<_, Transaction>(
        r#"
        select id, item_id, location_id, quantity, comment from transactions
        where id = $1
    "#,
    )
    .bind(id)
    .fetch_one(&*context.pool)
    .await
    .map_err(FieldError::from)
}

/// Creates an transaction, given an insertable transaction, returning the result, or a field error.
pub(crate) async fn create_transaction(
    context: &Context,
    transaction: InsertableTransaction,
) -> Result<Transaction, FieldError> {
    transaction.validate().map_err(FieldError::from)?;
    let result = sqlx::query_as::<_, Transaction>(
        r#"
        insert into transactions (item_id, location_id, quantity, comment)
        values ($1, $2, $3, $4)
        returning id, item_id, location_id, quantity, comment
    "#,
    )
    .bind(transaction.item_id)
    .bind(transaction.location_id)
    .bind(transaction.quantity)
    .bind(transaction.comment)
    .fetch_one(&*context.pool)
    .await
    .map_err(FieldError::from);

    // publish the created event using redis pubsub and send the created transaction data
    modification::broadcast(context, "transactions", ModificationType::Create, &result).await;

    result
}

/// Updates an transaction, given an insertable transaction, returning the result, or a field error.
pub(crate) async fn update_transaction(
    context: &Context,
    id: i32,
    transaction: InsertableTransaction,
) -> Result<Transaction, FieldError> {
    transaction.validate().map_err(FieldError::from)?;
    let result = sqlx::query_as::<_, Transaction>(
        r#"
        update transactions
        set item_id = $1, location_id = $2, quantity = $3, comment = $4
        where id = $5
        returning id, item_id, location_id, quantity, comment
    "#,
    )
    .bind(transaction.item_id)
    .bind(transaction.location_id)
    .bind(transaction.quantity)
    .bind(transaction.comment)
    .bind(id)
    .fetch_one(&*context.pool)
    .await
    .map_err(FieldError::from);

    // publish the updated event using redis pubsub and send the created transaction data
    modification::broadcast(context, "transactions", ModificationType::Update, &result).await;

    result
}

/// Deletes an transaction, given an id, returning the result, or a field error.
pub(crate) async fn delete_transaction(
    context: &Context,
    id: i32,
) -> Result<Transaction, FieldError> {
    let result = sqlx::query_as::<_, Transaction>(
        r#"
        delete from transactions
        where id = $1
        returning id, item_id, location_id, quantity, comment
    "#,
    )
    .bind(id)
    .fetch_one(&*context.pool)
    .await
    .map_err(FieldError::from);

    // publish the deleted event using redis pubsub and send the transaction data
    modification::broadcast(context, "transactions", ModificationType::Delete, &result).await;

    result
}

/// An transaction in the inventory tracking system.
#[graphql_object(context = Context)]
impl Transaction {
    /// The id of the transaction.
    fn id(&self) -> i32 {
        self.id
    }

    /// The item of the transaction.
    async fn item(&self, context: &Context) -> item::Item {
        item::get_item(context, self.item_id)
            .await
            .expect("dangling transaction has no item")
    }

    /// The location of the transaction.
    async fn location(&self, context: &Context) -> Option<location::Location> {
        match self.location_id {
            Some(location_id) => location::get_location(context, location_id).await.ok(),
            None => None,
        }
    }

    /// The change in quantity of items of the transaction.
    fn quantity(&self) -> i32 {
        self.quantity
    }

    /// The comment of the transaction.
    fn comment(&self) -> Option<&str> {
        self.comment.as_deref()
    }
}
