use juniper::FieldError;
use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::graphql::Context;
use crate::model::modification::{self, ModificationType};
use crate::model::transaction;

/// Item model returned by a query in the inventory tracking system.
#[derive(Debug, PartialEq, sqlx::FromRow, Serialize, Deserialize)]
pub(crate) struct Item {
    id: i32,
    sku: Option<String>,
    name: String,
    supplier: Option<String>,
    description: Option<String>,
}

/// Item model to insert in the inventory tracking system.
#[derive(Debug, PartialEq, Validate, Deserialize, GraphQLInputObject)]
#[graphql(description = "An item to input to the inventory system.")]
pub(crate) struct InsertableItem {
    #[validate(length(min = 1))]
    sku: Option<String>,
    #[validate(length(min = 1))]
    name: String,
    supplier: Option<String>,
    description: Option<String>,
}

#[derive(sqlx::FromRow, derive_more::Into)]
pub(crate) struct SqlSum(i64);

/// Gets all items, returning the result, or a field error.
pub(crate) async fn get_items(context: &Context) -> Result<Vec<Item>, FieldError> {
    sqlx::query_as::<_, Item>(
        r#"
        select id, sku, name, supplier, description from items
    "#,
    )
    .fetch_all(&*context.pool)
    .await
    .map_err(FieldError::from)
}

/// Gets an item, given an id, returning the result, or a field error.
pub(crate) async fn get_item(context: &Context, id: i32) -> Result<Item, FieldError> {
    sqlx::query_as::<_, Item>(
        r#"
        select id, sku, name, supplier, description from items
        where id = $1
    "#,
    )
    .bind(id)
    .fetch_one(&*context.pool)
    .await
    .map_err(FieldError::from)
}

/// Creates an item, given an insertable item, returning the result, or a field error.
pub(crate) async fn create_item(
    context: &Context,
    item: InsertableItem,
) -> Result<Item, FieldError> {
    item.validate().map_err(FieldError::from)?;
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
    .fetch_one(&*context.pool)
    .await
    .map_err(FieldError::from);

    // publish the created event using redis pubsub and send the created item data
    modification::broadcast(context, "items", ModificationType::Create, &result).await;

    result
}

/// Updates an item, given an insertable item, returning the result, or a field error.
pub(crate) async fn update_item(
    context: &Context,
    id: i32,
    item: InsertableItem,
) -> Result<Item, FieldError> {
    item.validate().map_err(FieldError::from)?;
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
    .fetch_one(&*context.pool)
    .await
    .map_err(FieldError::from);

    // publish the updated event using redis pubsub and send the created item data
    modification::broadcast(context, "items", ModificationType::Update, &result).await;

    result
}

/// Deletes an item, given an id, returning the result, or a field error.
pub(crate) async fn delete_item(context: &Context, id: i32) -> Result<Item, FieldError> {
    let result = sqlx::query_as::<_, Item>(
        r#"
        delete from items
        where id = $1
        returning id, sku, name, supplier, description
    "#,
    )
    .bind(id)
    .fetch_one(&*context.pool)
    .await
    .map_err(FieldError::from);

    // publish the deleted event using redis pubsub and send the item data
    modification::broadcast(context, "items", ModificationType::Delete, &result).await;

    result
}

/// Gets the quantity of the item at all locations.
pub(crate) async fn item_quantity(context: &Context, id: i32) -> Result<i32, FieldError> {
    let result = sqlx::query_as::<_, SqlSum>(r#"
        select coalesce(sum(quantity), 0) from (select quantity from transactions where item_id = $1) as item_transactions
    "#).bind(id)
        .fetch_one(&*context.pool).await;

    match result {
        Ok(quantity) => i32::try_from(i64::from(quantity)).map_err(FieldError::from),
        Err(e) => Err(FieldError::from(e)),
    }
}

/// Gets the transactions of the item.
pub(crate) async fn item_transactions(
    context: &Context,
    id: i32,
) -> Result<Vec<transaction::Transaction>, FieldError> {
    sqlx::query_as::<_, transaction::Transaction>(
        r#"
        select id, item_id, location_id, quantity, comment from transactions
        where item_id = $1
    "#,
    )
    .bind(id)
    .fetch_all(&*context.pool)
    .await
    .map_err(FieldError::from)
}

/// An item in the inventory tracking system.
#[graphql_object(context = Context)]
impl Item {
    /// The id of the item.
    fn id(&self) -> i32 {
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
    async fn quantity(&self, context: &Context) -> i32 {
        item_quantity(context, self.id).await.unwrap_or(0)
    }

    /// The transactions of the item.
    async fn transactions(&self, context: &Context) -> Vec<transaction::Transaction> {
        item_transactions(context, self.id).await.unwrap_or_default()
    }
}
