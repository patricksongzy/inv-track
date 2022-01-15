use std::collections::HashMap;
use std::fmt::Debug;

use async_graphql::{Error, Result};
use serde::{Deserialize, Serialize};

use crate::batcher::id_loader::IdLoader;
use crate::graphql::{AppContext, Clients};
use crate::model::modification::{self, ModificationType};
use crate::model::transaction::Transaction;
use crate::model::validation;

/// The id of an item.
#[derive(
    PartialEq,
    Eq,
    Into,
    Hash,
    Copy,
    Clone,
    Debug,
    sqlx::Type,
    Serialize,
    Deserialize,
)]
#[sqlx(transparent)]
pub(crate) struct ItemId(i32);
async_graphql::scalar!(ItemId);

/// The quantity of inventory.
#[derive(
    PartialEq, Into, Neg, Copy, Clone, Debug, sqlx::Type, Serialize, Deserialize,
)]
#[sqlx(transparent)]
pub(crate) struct ItemQuantity(i32);
async_graphql::scalar!(ItemQuantity);

/// Item model returned by a query in the inventory tracking system.
#[derive(Debug, Clone, PartialEq, sqlx::FromRow, Serialize, Deserialize, async_graphql::SimpleObject)]
#[graphql(complex)]
pub(crate) struct Item {
    id: ItemId,
    sku: Option<String>,
    name: String,
    supplier: Option<String>,
    description: Option<String>,
}

/// Item model to input to the inventory tracking system.
#[derive(Debug, PartialEq, Deserialize, async_graphql::InputObject)]
pub(crate) struct InsertableItem {
    #[graphql(validator(min_length = 1))]
    pub(crate) sku: Option<String>,
    #[graphql(validator(min_length = 1))]
    name: String,
    supplier: Option<String>,
    description: Option<String>,
}

/// Gets all items, returning the result, or an error error.
pub(crate) async fn get_items(context: &AppContext) -> Result<Vec<Item>> {
    sqlx::query_as::<_, Item>(
        r#"
        select id, sku, name, supplier, description from items
    "#,
    )
    .fetch_all(&*context.clients.postgres)
    .await
    .map_err(Error::from)
}

/// Gets all items with the given ids.
pub(crate) async fn get_items_by_ids(
    clients: &Clients,
    ids: Vec<ItemId>,
) -> Result<HashMap<ItemId, Item>> {
    sqlx::query_as::<_, Item>(
        r#"
        select id, sku, name, supplier, description from items
        where id = any($1)
    "#,
    )
    .bind(ids.into_iter().map(|id| id.0).collect::<Vec<i32>>())
    .fetch_all(&*clients.postgres)
    .await
    .map(|items| items.into_iter().map(|item| (item.id, item)).collect())
    .map_err(Error::from)
}

/// Gets all transactions with the given item ids.
pub(crate) async fn get_transactions_by_item_ids(
    clients: &Clients,
    ids: Vec<ItemId>,
) -> Result<HashMap<ItemId, Vec<Transaction>>> {
    sqlx::query_as::<_, Transaction>(
        r#"
        select id, item_id, location_id, transaction_date, quantity, comment from transactions
        where item_id = any($1)
    "#,
    )
    .bind(ids.into_iter().map(|id| id.0).collect::<Vec<i32>>())
    .fetch_all(&*clients.postgres)
    .await
    .map(|transactions| {
        let mut transactions_map = HashMap::new();
        transactions.into_iter().for_each(|transaction| {
            transactions_map
                .entry(transaction.item_id)
                .or_insert(Vec::new())
                .push(transaction);
        });
        transactions_map
    })
    .map_err(Error::from)
}

/// Gets the item quantities for items with the given item ids.
pub(crate) async fn get_quantities_by_item_ids(
    clients: &Clients,
    ids: Vec<ItemId>,
) -> Result<HashMap<ItemId, ItemQuantity>> {
    let results = sqlx::query!(
        r#"
        select item_id, coalesce(sum(quantity), 0) from transactions
        where item_id = any($1)
        group by item_id
    "#,
        &ids.into_iter().map(|id| id.0).collect::<Vec<i32>>()
    )
    .fetch_all(&*clients.postgres)
    .await
    .map_err(Error::from)?;

    let mut results_map = HashMap::new();
    for result in results {
        results_map.insert(
            ItemId(result.item_id),
            ItemQuantity(i32::try_from(result.coalesce.unwrap_or(0))?),
        );
    }
    Ok(results_map)
}

/// Gets an item, given an id, returning the result, or an error.
pub(crate) async fn get_item(context: &AppContext, id: ItemId) -> Result<Item> {
    context
        .loaders
        .get::<IdLoader<ItemId, Item, Clients>>()
        .unwrap()
        .load(id)
        .await
}

/// Creates an item, given an insertable item, returning the result, or an error.
pub(crate) async fn create_item(context: &AppContext, item: InsertableItem) -> Result<Item> {
    // check that the sku is unique
    validation::item::validate_sku(context, &item).await?;

    let created = sqlx::query_as::<_, Item>(
        r#"
        insert into items (sku, name, supplier, description)
        values ($1, $2, $3, $4)
        returning id, sku, name, supplier, description
    "#,
    )
    .bind(item.sku)
    .bind(item.name)
    .bind(item.supplier)
    .bind(item.description)
    .fetch_one(&*context.clients.postgres)
    .await
    .map_err(Error::from)?;

    // publish the created event using redis pubsub and send the created item data
    modification::broadcast(context, "items", ModificationType::Create, &created).await;

    Ok(created)
}

/// Updates an item, given an insertable item, returning the result, or an error.
pub(crate) async fn update_item(
    context: &AppContext,
    id: ItemId,
    item: InsertableItem,
) -> Result<Item, Error> {
    let updated = sqlx::query_as::<_, Item>(
        r#"
        update items
        set sku = $1, name = $2, supplier = $3, description = $4
        where id = $5
        returning id, sku, name, supplier, description
    "#,
    )
    .bind(item.sku)
    .bind(item.name)
    .bind(item.supplier)
    .bind(item.description)
    .bind(id)
    .fetch_one(&*context.clients.postgres)
    .await
    .map_err(Error::from)?;

    // publish the updated event using redis pubsub and send the item data
    modification::broadcast(context, "items", ModificationType::Update, &updated).await;

    Ok(updated)
}

/// Deletes an item, given an id, returning the result, or an error.
pub(crate) async fn delete_item(context: &AppContext, id: ItemId) -> Result<Item> {
    let deleted = sqlx::query_as::<_, Item>(
        r#"
        delete from items
        where id = $1
        returning id, sku, name, supplier, description
    "#,
    )
    .bind(id)
    .fetch_one(&*context.clients.postgres)
    .await
    .map_err(Error::from)?;

    // publish the deleted event using redis pubsub and send the item data
    modification::broadcast(context, "items", ModificationType::Delete, &deleted).await;

    Ok(deleted)
}

/// An item in the inventory tracking system.
#[async_graphql::ComplexObject]
impl Item {
    /// The quantity of the item.
    async fn quantity(&self, context: &async_graphql::Context<'_>) -> ItemQuantity {
        context.data_unchecked::<AppContext>()
            .loaders
            .get::<IdLoader<ItemId, ItemQuantity, Clients>>()
            .unwrap()
            .load(self.id)
            .await
            .unwrap_or(ItemQuantity(0))
    }

    /// The transactions of the item.
    async fn transactions(&self, context: &async_graphql::Context<'_>) -> Vec<Transaction> {
        context.data_unchecked::<AppContext>()
            .loaders
            .get::<IdLoader<ItemId, Vec<Transaction>, Clients>>()
            .unwrap()
            .load(self.id)
            .await
            .unwrap_or_default()
    }
}
