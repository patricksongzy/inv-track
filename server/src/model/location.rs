use juniper::FieldError;
use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::graphql::Context;
use crate::model::modification::{self, ModificationType};
use crate::model::transaction;

/// Location model returned by a query in the inventory tracking system.
#[derive(Debug, PartialEq, sqlx::FromRow, Serialize, Deserialize)]
pub(crate) struct Location {
    id: i32,
    name: String,
    address: Option<String>,
}

/// Location model to insert in the inventory tracking system.
#[derive(Debug, PartialEq, Validate, Deserialize, GraphQLInputObject)]
#[graphql(description = "A location to input to the inventory system.")]
pub(crate) struct InsertableLocation {
    #[validate(length(min = 1))]
    name: String,
    #[validate(length(min = 1))]
    address: Option<String>,
}

/// Gets all locations, returning the result, or a field error.
pub(crate) async fn get_locations(context: &Context) -> Result<Vec<Location>, FieldError> {
    sqlx::query_as::<_, Location>(
        r#"
        select id, name, address from locations
    "#,
    )
    .fetch_all(&*context.pool)
    .await
    .map_err(FieldError::from)
}

/// Gets an location, given an id, returning the result, or a field error.
pub(crate) async fn get_location(context: &Context, id: i32) -> Result<Location, FieldError> {
    sqlx::query_as::<_, Location>(
        r#"
        select id, name, address from locations
        where id = $1
    "#,
    )
    .bind(id)
    .fetch_one(&*context.pool)
    .await
    .map_err(FieldError::from)
}

/// Creates an location, given an insertable location, returning the result, or a field error.
pub(crate) async fn create_location(
    context: &Context,
    location: InsertableLocation,
) -> Result<Location, FieldError> {
    location.validate().map_err(FieldError::from)?;
    let result = sqlx::query_as::<_, Location>(
        r#"
        insert into locations (name, address)
        values ($1, $2)
        returning id, name, address
    "#,
    )
    .bind(location.name)
    .bind(location.address)
    .fetch_one(&*context.pool)
    .await
    .map_err(FieldError::from);

    // publish the created event using redis pubsub and send the created location data
    modification::broadcast(context, "locations", ModificationType::Create, &result).await;

    result
}

/// Updates an location, given an insertable location, returning the result, or a field error.
pub(crate) async fn update_location(
    context: &Context,
    id: i32,
    location: InsertableLocation,
) -> Result<Location, FieldError> {
    location.validate().map_err(FieldError::from)?;
    let result = sqlx::query_as::<_, Location>(
        r#"
        update locations
        set name = $1, address = $2
        where id = $3
        returning id, name, address
    "#,
    )
    .bind(location.name)
    .bind(location.address)
    .bind(id)
    .fetch_one(&*context.pool)
    .await
    .map_err(FieldError::from);

    // publish the updated event using redis pubsub and send the created location data
    modification::broadcast(context, "locations", ModificationType::Update, &result).await;

    result
}

/// Deletes an location, given an id, returning the result, or a field error.
pub(crate) async fn delete_location(context: &Context, id: i32) -> Result<Location, FieldError> {
    let result = sqlx::query_as::<_, Location>(
        r#"
        delete from locations
        where id = $1
        returning id, name, address
    "#,
    )
    .bind(id)
    .fetch_one(&*context.pool)
    .await
    .map_err(FieldError::from);

    // publish the deleted event using redis pubsub and send the location data
    modification::broadcast(context, "locations", ModificationType::Delete, &result).await;

    result
}

/// Gets the transactions at the location.
pub(crate) async fn location_transactions(
    context: &Context,
    id: i32,
) -> Result<Vec<transaction::Transaction>, FieldError> {
    sqlx::query_as::<_, transaction::Transaction>(
        r#"
        select id, item_id, location_id, quantity, comment from transactions
        where location_id = $1
    "#,
    )
    .bind(id)
    .fetch_all(&*context.pool)
    .await
    .map_err(FieldError::from)
}

/// An location in the inventory tracking system.
#[graphql_object(context = Context)]
impl Location {
    /// The id of the location.
    fn id(&self) -> i32 {
        self.id
    }

    /// The name of the location.
    fn name(&self) -> &str {
        &self.name
    }

    /// The address of the location.
    fn address(&self) -> Option<&str> {
        self.address.as_deref()
    }

    /// The transactions at the location.
    async fn transactions(&self, context: &Context) -> Vec<transaction::Transaction> {
        location_transactions(context, self.id)
            .await
            .unwrap_or_default()
    }
}
