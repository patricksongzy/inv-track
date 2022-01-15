use std::collections::HashMap;
use std::fmt::Debug;

use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::batcher::id_loader::IdLoader;
use crate::error::AppError;
use crate::graphql::{Clients, Context};
use crate::model::item::{self, Item, ItemId, ItemQuantity};
use crate::model::location::{self, Location, LocationId};
use crate::model::modification::{self, ModificationType};
use crate::model::validation;

/// The id of a transaction.
#[derive(
    PartialEq,
    Eq,
    Into,
    Hash,
    Copy,
    Clone,
    Debug,
    GraphQLScalarValue,
    sqlx::Type,
    Serialize,
    Deserialize,
)]
#[sqlx(transparent)]
pub(crate) struct TransactionId(i32);

/// Transaction model returned by a query in the inventory tracking system.
#[derive(Debug, Clone, PartialEq, sqlx::FromRow, Serialize, Deserialize)]
pub(crate) struct Transaction {
    id: TransactionId,
    pub(crate) item_id: ItemId,
    pub(crate) location_id: Option<LocationId>,
    quantity: ItemQuantity,
    comment: Option<String>,
}

/// Transaction model to insert in the inventory tracking system.
#[derive(Debug, PartialEq, Validate, Deserialize, GraphQLInputObject)]
#[graphql(description = "A transaction to input to the inventory system.")]
pub(crate) struct InsertableTransaction {
    #[serde(rename = "itemId")]
    pub(crate) item_id: ItemId,
    #[serde(rename = "locationId")]
    pub(crate) location_id: Option<LocationId>,
    quantity: ItemQuantity,
    comment: Option<String>,
}

/// Gets all transactions, returning the result, or a field error.
pub(crate) async fn get_transactions(context: &Context) -> Result<Vec<Transaction>, AppError> {
    sqlx::query_as::<_, Transaction>(
        r#"
        select id, item_id, location_id, quantity, comment from transactions
    "#,
    )
    .fetch_all(&*context.clients.postgres)
    .await
    .map_err(AppError::from)
}

/// Gets all transactions with the given ids.
pub(crate) async fn get_transactions_by_ids(
    clients: &Clients,
    ids: Vec<TransactionId>,
) -> Result<HashMap<TransactionId, Transaction>, AppError> {
    sqlx::query_as::<_, Transaction>(
        r#"
        select id, item_id, location_id, quantity, comment from transactions
        where id = any($1)
    "#,
    )
    .bind(ids.into_iter().map(|id| id.0).collect::<Vec<i32>>())
    .fetch_all(&*clients.postgres)
    .await
    .map(|transactions| {
        transactions
            .into_iter()
            .map(|transaction| (transaction.id, transaction))
            .collect()
    })
    .map_err(AppError::from)
}

/// Gets an transaction, given an id, returning the result, or a field error.
pub(crate) async fn get_transaction(
    context: &Context,
    id: TransactionId,
) -> Result<Transaction, AppError> {
    context
        .loaders
        .get::<IdLoader<TransactionId, Transaction, Clients>>()
        .unwrap()
        .load(id)
        .await
}

/// Creates an transaction, given an insertable transaction, returning the result, or a field error.
pub(crate) async fn create_transaction(
    context: &Context,
    transaction: InsertableTransaction,
) -> Result<Transaction, AppError> {
    transaction.validate().map_err(AppError::from_validation)?;

    // check that the item and location exist
    validation::transaction::validate_ids(context, &transaction).await?;

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
    .fetch_one(&*context.clients.postgres)
    .await
    .map_err(AppError::from);

    // publish the created event using redis pubsub and send the created transaction data
    modification::broadcast(context, "transactions", ModificationType::Create, &result).await;

    result
}

/// Updates an transaction, given an insertable transaction, returning the result, or a field error.
pub(crate) async fn update_transaction(
    context: &Context,
    id: TransactionId,
    transaction: InsertableTransaction,
) -> Result<Transaction, AppError> {
    transaction.validate().map_err(AppError::from_validation)?;

    // check that the item and location exist
    validation::transaction::validate_ids(context, &transaction).await?;

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
    .fetch_one(&*context.clients.postgres)
    .await
    .map_err(AppError::from);

    // publish the updated event using redis pubsub and send the created transaction data
    modification::broadcast(context, "transactions", ModificationType::Update, &result).await;

    result
}

/// Deletes an transaction, given an id, returning the result, or a field error.
pub(crate) async fn delete_transaction(
    context: &Context,
    id: TransactionId,
) -> Result<Transaction, AppError> {
    let result = sqlx::query_as::<_, Transaction>(
        r#"
        delete from transactions
        where id = $1
        returning id, item_id, location_id, quantity, comment
    "#,
    )
    .bind(id)
    .fetch_one(&*context.clients.postgres)
    .await
    .map_err(AppError::from);

    // publish the deleted event using redis pubsub and send the transaction data
    modification::broadcast(context, "transactions", ModificationType::Delete, &result).await;

    result
}

/// An transaction in the inventory tracking system.
#[graphql_object(context = Context)]
impl Transaction {
    /// The id of the transaction.
    fn id(&self) -> TransactionId {
        self.id
    }

    /// The item of the transaction.
    async fn item(&self, context: &Context) -> Item {
        item::get_item(context, self.item_id)
            .await
            .expect("dangling transaction has no item")
    }

    /// The location of the transaction.
    async fn location(&self, context: &Context) -> Option<Location> {
        match self.location_id {
            Some(location_id) => location::get_location(context, location_id).await.ok(),
            None => None,
        }
    }

    /// The change in quantity of items of the transaction.
    fn quantity(&self) -> ItemQuantity {
        self.quantity
    }

    /// The comment of the transaction.
    fn comment(&self) -> Option<&str> {
        self.comment.as_deref()
    }
}
