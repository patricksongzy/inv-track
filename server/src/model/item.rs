use std::collections::HashMap;
use std::fmt::Debug;

use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::error::AppError;
use crate::graphql::{Clients, Context};
use crate::batcher::id_loader::IdLoader;
use crate::model::modification::{self, ModificationType};
use crate::model::transaction::Transaction;

// TODO LIST
// 1. testing
// 2. validation before db
//   * check transaction item_id, location_id exists
//   * check item sku is unique
//   * check quantity is positive after transaction
// 3. front end

/// The id of an item.
#[derive(GraphQLScalarValue, PartialEq, Eq, Hash, Copy, Clone, Debug, sqlx::Type, Serialize, Deserialize)]
#[sqlx(transparent)]
pub(crate) struct ItemId(i32);
/// The quantity of inventory.
#[derive(GraphQLScalarValue, PartialEq, Copy, Clone, Debug, sqlx::Type, Serialize, Deserialize)]
#[sqlx(transparent)]
pub(crate) struct ItemQuantity(i32);

/// Item model returned by a query in the inventory tracking system.
#[derive(Debug, Clone, PartialEq, sqlx::FromRow, Serialize, Deserialize)]
pub(crate) struct Item {
    id: ItemId,
    sku: Option<String>,
    name: String,
    supplier: Option<String>,
    description: Option<String>,
}

/// Item model to insert in the inventory tracking system.
#[derive(Debug, PartialEq, Validate, Deserialize, GraphQLInputObject)]
#[graphql(description = "An item to input to the inventory system.")]
pub(crate) struct InsertableItem {
    #[validate(length(min = 1, message = "sku must be at least 1 character long"))]
    sku: Option<String>,
    #[validate(length(min = 1, message = "name must be at least 1 character long"))]
    name: String,
    supplier: Option<String>,
    description: Option<String>,
}

/// Gets all items, returning the result, or a field error.
pub(crate) async fn get_items(context: &Context) -> Result<Vec<Item>, AppError> {
    sqlx::query_as::<_, Item>(
        r#"
        select id, sku, name, supplier, description from items
    "#,
    )
    .fetch_all(&*context.clients.postgres)
    .await
    .map_err(AppError::from)
}

/// Gets all items with the given ids.
pub(crate) async fn get_items_by_ids(
    clients: &Clients,
    ids: Vec<ItemId>,
) -> Result<HashMap<ItemId, Item>, AppError> {
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
    .map_err(AppError::from)
}

/// Gets all transactions with the given item ids.
pub(crate) async fn get_transactions_by_item_ids(
    clients: &Clients,
    ids: Vec<ItemId>,
) -> Result<HashMap<ItemId, Vec<Transaction>>, AppError> {
    sqlx::query_as::<_, Transaction>(
        r#"
        select id, item_id, location_id, quantity, comment from transactions
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
    .map_err(AppError::from)
}

/// Gets the item quantities for items with the given item ids.
pub(crate) async fn get_quantities_by_item_ids(
    clients: &Clients,
    ids: Vec<ItemId>,
) -> Result<HashMap<ItemId, ItemQuantity>, AppError> {
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
    .map_err(AppError::from)?;

    let mut results_map = HashMap::new();
    for result in results {
        results_map.insert(
            ItemId(result.item_id),
            ItemQuantity(i32::try_from(result.coalesce.unwrap_or(0))?),
        );
    }
    Ok(results_map)
}

/// Gets an item, given an id, returning the result, or a field error.
pub(crate) async fn get_item(context: &Context, id: ItemId) -> Result<Item, AppError> {
    context.loaders.get::<IdLoader<ItemId, Item>>().unwrap().load(id).await
}

/// Creates an item, given an insertable item, returning the result, or a field error.
pub(crate) async fn create_item(context: &Context, item: InsertableItem) -> Result<Item, AppError> {
    item.validate().map_err(AppError::from_validation)?;
    let result = sqlx::query_as::<_, Item>(
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
    .map_err(AppError::from);

    // publish the created event using redis pubsub and send the created item data
    modification::broadcast(context, "items", ModificationType::Create, &result).await;

    result
}

/// Updates an item, given an insertable item, returning the result, or a field error.
pub(crate) async fn update_item(
    context: &Context,
    id: ItemId,
    item: InsertableItem,
) -> Result<Item, AppError> {
    item.validate().map_err(AppError::from_validation)?;
    let result = sqlx::query_as::<_, Item>(
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
    .map_err(AppError::from);

    // publish the updated event using redis pubsub and send the created item data
    modification::broadcast(context, "items", ModificationType::Update, &result).await;

    result
}

/// Deletes an item, given an id, returning the result, or a field error.
pub(crate) async fn delete_item(context: &Context, id: ItemId) -> Result<Item, AppError> {
    let result = sqlx::query_as::<_, Item>(
        r#"
        delete from items
        where id = $1
        returning id, sku, name, supplier, description
    "#,
    )
    .bind(id)
    .fetch_one(&*context.clients.postgres)
    .await
    .map_err(AppError::from);

    // publish the deleted event using redis pubsub and send the item data
    modification::broadcast(context, "items", ModificationType::Delete, &result).await;

    result
}

/// An item in the inventory tracking system.
#[graphql_object(context = Context)]
impl Item {
    /// The id of the item.
    fn id(&self) -> ItemId {
        self.id
    }

    /// The stock-keeping unit of the item.
    fn sku(&self) -> Option<&str> {
        self.sku.as_deref()
    }

    /// The name of the item.
    fn name(&self) -> &str {
        &self.name
    }

    /// The supplier of the item.
    fn supplier(&self) -> Option<&str> {
        self.supplier.as_deref()
    }

    /// The description of the item.
    fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    /// The quantity of the item.
    async fn quantity(&self, context: &Context) -> ItemQuantity {
        context.loaders.get::<IdLoader<ItemId, ItemQuantity>>().unwrap().load(self.id).await.unwrap_or(ItemQuantity(0))
    }

    /// The transactions of the item.
    async fn transactions(&self, context: &Context) -> Vec<Transaction> {
        context.loaders.get::<IdLoader<ItemId, Vec<Transaction>>>().unwrap().load(self.id).await.unwrap_or_default()
    }
}
