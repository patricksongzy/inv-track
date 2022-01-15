use std::borrow::Cow;

use serde::Serialize;
use validator::{ValidationError, ValidationErrors};

use crate::error::AppError;
use crate::graphql::Context;

fn add_error<T: Serialize>(
    errors: &mut ValidationErrors,
    code: &'static str,
    message: &'static str,
    field: &'static str,
    value: T,
) {
    let mut error = ValidationError::new(code);
    error.message = Some(Cow::Borrowed(message));
    error.add_param(Cow::Borrowed("value"), &value);
    errors.add(field, error);
}

pub(crate) mod transaction {
    use super::*;
    use crate::model::transaction::InsertableTransaction;

    /// Validates that the item and location for a transaction exist.
    pub(crate) async fn validate_ids(
        context: &Context,
        transaction: &InsertableTransaction,
    ) -> Result<(), AppError> {
        let mut errors = ValidationErrors::new();

        // check item exists
        let item_count = sqlx::query!(
            r#"select count(id) from items where id = $1"#,
            i32::from(transaction.item_id)
        )
        .fetch_one(&*context.clients.postgres)
        .await
        .map_err(AppError::from)?
        .count
        .unwrap_or(0);

        if item_count != 1 {
            add_error(
                &mut errors,
                "existence",
                "item not found",
                "itemId",
                i32::from(transaction.item_id),
            );
        }

        // check location exists
        if let Some(location_id) = transaction.location_id {
            let location_count = sqlx::query!(
                r#"select count(id) from locations where id = $1"#,
                i32::from(location_id)
            )
            .fetch_one(&*context.clients.postgres)
            .await
            .map_err(AppError::from)?
            .count
            .unwrap_or(0);

            if location_count != 1 {
                add_error(
                    &mut errors,
                    "existence",
                    "location not found",
                    "locationId",
                    i32::from(location_id),
                );
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(AppError::from_validation(errors))
        }
    }
}

pub(crate) mod item {
    use super::*;
    use crate::model::item::InsertableItem;

    pub(crate) async fn validate_sku(
        context: &Context,
        item: &InsertableItem,
    ) -> Result<(), AppError> {
        let mut errors = ValidationErrors::new();

        if let Some(sku) = &item.sku {
            let sku_count = sqlx::query!(
                r#"select count(id) from items where upper(sku) = upper($1)"#,
                sku
            )
            .fetch_one(&*context.clients.postgres)
            .await
            .map_err(AppError::from)?
            .count
            .unwrap_or(0);

            if sku_count == 0 {
                Ok(())
            } else {
                add_error(&mut errors, "uniqueness", "sku is not unique", "sku", sku);
                Err(AppError::from_validation(errors))
            }
        } else {
            Ok(())
        }
    }
}
