use std::collections::HashMap;
use std::fmt::Debug;

use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::batcher::id_loader::IdLoader;
use crate::error::AppError;
use crate::graphql::{Clients, Context};
use crate::model::modification::{self, ModificationType};
use crate::model::transaction::Transaction;

/// The id of a location.
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
pub(crate) struct LocationId(i32);

/// Location model returned by a query in the inventory tracking system.
#[derive(Debug, Clone, PartialEq, sqlx::FromRow, Serialize, Deserialize)]
pub(crate) struct Location {
    id: LocationId,
    name: String,
    address: Option<String>,
}

/// Location model to insert in the inventory tracking system.
#[derive(Debug, PartialEq, Validate, Deserialize, GraphQLInputObject)]
#[graphql(description = "A location to input to the inventory system.")]
pub(crate) struct InsertableLocation {
    #[validate(length(min = 1, message = "name must be at least 1 character long"))]
    name: String,
    #[validate(length(min = 1, message = "address must be at least 1 character long"))]
    address: Option<String>,
}

/// Gets all locations, returning the result, or a field error.
pub(crate) async fn get_locations(context: &Context) -> Result<Vec<Location>, AppError> {
    sqlx::query_as::<_, Location>(
        r#"
        select id, name, address from locations
    "#,
    )
    .fetch_all(&*context.clients.postgres)
    .await
    .map_err(AppError::from)
}

/// Gets all locations with the given ids.
pub(crate) async fn get_locations_by_ids(
    clients: &Clients,
    ids: Vec<LocationId>,
) -> Result<HashMap<LocationId, Location>, AppError> {
    sqlx::query_as::<_, Location>(
        r#"
        select id, name, address from locations
        where id = any($1)
    "#,
    )
    .bind(ids.into_iter().map(|id| id.0).collect::<Vec<i32>>())
    .fetch_all(&*clients.postgres)
    .await
    .map(|locations| {
        locations
            .into_iter()
            .map(|location| (location.id, location))
            .collect()
    })
    .map_err(AppError::from)
}

/// Gets all transactions with the given location ids.
pub(crate) async fn get_transactions_by_location_ids(
    clients: &Clients,
    ids: Vec<LocationId>,
) -> Result<HashMap<LocationId, Vec<Transaction>>, AppError> {
    sqlx::query_as::<_, Transaction>(
        r#"
        select id, item_id, location_id, transaction_date, quantity, comment from transactions
        where location_id = any($1)
    "#,
    )
    .bind(ids.into_iter().map(|id| id.0).collect::<Vec<i32>>())
    .fetch_all(&*clients.postgres)
    .await
    .map(|transactions| {
        let mut transactions_map = HashMap::new();
        transactions.into_iter().for_each(|transaction| {
            transactions_map
                .entry(transaction.location_id.unwrap())
                .or_insert(Vec::new())
                .push(transaction);
        });
        transactions_map
    })
    .map_err(AppError::from)
}

/// Gets an location, given an id, returning the result, or a field error.
pub(crate) async fn get_location(context: &Context, id: LocationId) -> Result<Location, AppError> {
    context
        .loaders
        .get::<IdLoader<LocationId, Location, Clients>>()
        .unwrap()
        .load(id)
        .await
}

/// Creates an location, given an insertable location, returning the result, or a field error.
pub(crate) async fn create_location(
    context: &Context,
    location: InsertableLocation,
) -> Result<Location, AppError> {
    location.validate().map_err(AppError::from_validation)?;
    let created = sqlx::query_as::<_, Location>(
        r#"
        insert into locations (name, address)
        values ($1, $2)
        returning id, name, address
    "#,
    )
    .bind(location.name)
    .bind(location.address)
    .fetch_one(&*context.clients.postgres)
    .await
    .map_err(AppError::from)?;

    // publish the created event using redis pubsub and send the created location data
    modification::broadcast(context, "locations", ModificationType::Create, &created).await;

    Ok(created)
}

/// Updates an location, given an insertable location, returning the result, or a field error.
pub(crate) async fn update_location(
    context: &Context,
    id: LocationId,
    location: InsertableLocation,
) -> Result<Location, AppError> {
    location.validate().map_err(AppError::from_validation)?;
    let updated = sqlx::query_as::<_, Location>(
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
    .fetch_one(&*context.clients.postgres)
    .await
    .map_err(AppError::from)?;

    // publish the updated event using redis pubsub and send the created location data
    modification::broadcast(context, "locations", ModificationType::Update, &updated).await;

    Ok(updated)
}

/// Deletes an location, given an id, returning the result, or a field error.
pub(crate) async fn delete_location(
    context: &Context,
    id: LocationId,
) -> Result<Location, AppError> {
    let deleted = sqlx::query_as::<_, Location>(
        r#"
        delete from locations
        where id = $1
        returning id, name, address
    "#,
    )
    .bind(id)
    .fetch_one(&*context.clients.postgres)
    .await
    .map_err(AppError::from)?;

    // publish the deleted event using redis pubsub and send the location data
    modification::broadcast(context, "locations", ModificationType::Delete, &deleted).await;

    Ok(deleted)
}

/// An location in the inventory tracking system.
#[graphql_object(context = Context)]
impl Location {
    /// The id of the location.
    fn id(&self) -> LocationId {
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
    async fn transactions(&self, context: &Context) -> Vec<Transaction> {
        context
            .loaders
            .get::<IdLoader<LocationId, Vec<Transaction>, Clients>>()
            .unwrap()
            .load(self.id)
            .await
            .unwrap_or_default()
    }
}
