use std::collections::HashMap;
use std::fmt::Debug;

use async_graphql::{Error, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::batcher::id_loader::IdLoader;
use crate::graphql::{AppContext, Clients};
use crate::model::item::{self, Item, ItemId, ItemQuantity};
use crate::model::location::{self, Location, LocationId};
use crate::model::modification::{self, ModificationType};
use crate::model::validation;

/// The id of a transaction.
#[derive(PartialEq, Eq, Into, Hash, Copy, Clone, Debug, sqlx::Type, Serialize, Deserialize)]
#[sqlx(transparent)]
pub(crate) struct TransactionId(i32);
async_graphql::scalar!(TransactionId);

/// Transaction model returned by a query in the inventory tracking system.
#[derive(
    Debug, Clone, PartialEq, sqlx::FromRow, Serialize, Deserialize, async_graphql::SimpleObject,
)]
#[graphql(complex)]
pub(crate) struct Transaction {
    id: TransactionId,
    pub(crate) item_id: ItemId,
    pub(crate) location_id: Option<LocationId>,
    transaction_date: Option<DateTime<Utc>>,
    quantity: ItemQuantity,
    comment: Option<String>,
}

/// Transaction model to input to the inventory tracking system.
#[derive(Debug, PartialEq, Deserialize, async_graphql::InputObject)]
pub(crate) struct InsertableTransaction {
    #[serde(rename = "itemId")]
    pub(crate) item_id: ItemId,
    #[serde(rename = "locationId")]
    pub(crate) location_id: Option<LocationId>,
    /// The date in RFC 3339 format.
    transaction_date: Option<DateTime<Utc>>,
    #[graphql(validator(custom = "validation::transaction::TransactionQuantityValidator {}"))]
    quantity: ItemQuantity,
    #[graphql(validator(min_length = 1))]
    comment: Option<String>,
}

/// Gets all transactions, returning the result, or a field error.
pub(crate) async fn get_transactions(context: &AppContext) -> Result<Vec<Transaction>> {
    sqlx::query_as::<_, Transaction>(
        r#"
        select id, item_id, location_id, transaction_date, quantity, comment from transactions
        order by transaction_date desc
    "#,
    )
    .fetch_all(&*context.clients.postgres)
    .await
    .map_err(Error::from)
}

/// Gets all transactions with the given ids.
pub(crate) async fn get_transactions_by_ids(
    clients: &Clients,
    ids: Vec<TransactionId>,
) -> Result<HashMap<TransactionId, Transaction>> {
    sqlx::query_as::<_, Transaction>(
        r#"
        select id, item_id, location_id, transaction_date, quantity, comment from transactions
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
    .map_err(Error::from)
}

/// Gets an transaction, given an id, returning the result, or a field error.
pub(crate) async fn get_transaction(
    context: &AppContext,
    id: TransactionId,
) -> Result<Transaction> {
    context
        .loaders
        .get::<IdLoader<TransactionId, Transaction, Clients>>()
        .unwrap()
        .load(id)
        .await
}

/// Creates an transaction, given an insertable transaction, returning the result, or a field error.
pub(crate) async fn create_transaction(
    context: &AppContext,
    transaction: InsertableTransaction,
) -> Result<Transaction> {
    // check that the item and location exist
    validation::transaction::validate_ids(context, &transaction).await?;

    let created = sqlx::query_as::<_, Transaction>(
        r#"
        insert into transactions (item_id, location_id, transaction_date, quantity, comment)
        values ($1, $2, $3, $4, $5)
        returning id, item_id, location_id, transaction_date, quantity, comment
    "#,
    )
    .bind(transaction.item_id)
    .bind(transaction.location_id)
    .bind(transaction.transaction_date)
    .bind(transaction.quantity)
    .bind(transaction.comment)
    .fetch_one(&*context.clients.postgres)
    .await
    .map_err(Error::from)?;

    // publish the created event using redis pubsub and send the created transaction data
    created
        .broadcast_update(context, ModificationType::Create)
        .await;

    Ok(created)
}

/// Updates an transaction, given an insertable transaction, returning the result, or a field error.
pub(crate) async fn update_transaction(
    context: &AppContext,
    id: TransactionId,
    transaction: InsertableTransaction,
) -> Result<Transaction> {
    // check that the item and location exist
    validation::transaction::validate_ids(context, &transaction).await?;

    let updated = sqlx::query_as::<_, Transaction>(
        r#"
        update transactions
        set item_id = $1, location_id = $2, transaction_date = $3, quantity = $4, comment = $5
        where id = $6
        returning id, item_id, location_id, quantity, transaction_date, comment
    "#,
    )
    .bind(transaction.item_id)
    .bind(transaction.location_id)
    .bind(transaction.transaction_date)
    .bind(transaction.quantity)
    .bind(transaction.comment)
    .bind(id)
    .fetch_one(&*context.clients.postgres)
    .await
    .map_err(Error::from)?;

    // publish the deleted event using redis pubsub and send the transaction data
    updated
        .broadcast_update(context, ModificationType::Update)
        .await;

    Ok(updated)
}

/// Deletes an transaction, given an id, returning the result, or a field error.
pub(crate) async fn delete_transaction(
    context: &AppContext,
    id: TransactionId,
) -> Result<Transaction> {
    let deleted = sqlx::query_as::<_, Transaction>(
        r#"
        delete from transactions
        where id = $1
        returning id, item_id, location_id, transaction_date, quantity, comment
    "#,
    )
    .bind(id)
    .fetch_one(&*context.clients.postgres)
    .await
    .map_err(Error::from)?;

    // publish the deleted event using redis pubsub and send the transaction data
    deleted
        .broadcast_update(context, ModificationType::Delete)
        .await;

    Ok(deleted)
}

impl Transaction {
    async fn get_item(&self, context: &AppContext) -> Option<Item> {
        item::get_item(context, self.item_id).await.ok()
    }

    async fn get_location(&self, context: &AppContext) -> Option<Location> {
        match self.location_id {
            Some(location_id) => location::get_location(context, location_id).await.ok(),
            None => None,
        }
    }

    async fn broadcast_update(&self, context: &AppContext, modification: ModificationType) {
        // publish the event using redis pubsub and send the transaction data
        modification::broadcast(context, "transactions", modification, self).await;
        if let Some(item) = self.get_item(context).await {
            modification::broadcast(context, "items", ModificationType::Update, &item).await;
        }
        if let Some(location) = self.get_location(context).await {
            modification::broadcast(context, "locations", ModificationType::Update, &location)
                .await;
        }
    }
}

/// An transaction in the inventory tracking system.
#[async_graphql::ComplexObject]
impl Transaction {
    /// The item of the transaction.
    async fn item(&self, context: &async_graphql::Context<'_>) -> Item {
        self.get_item(context.data_unchecked::<AppContext>())
            .await
            .expect("dangling transaction has no item")
    }

    /// The location of the transaction.
    async fn location(&self, context: &async_graphql::Context<'_>) -> Option<Location> {
        self.get_location(context.data_unchecked::<AppContext>())
            .await
    }
}
