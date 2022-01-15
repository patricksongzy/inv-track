use std::fmt::Display;

use juniper::{FieldError, IntoFieldError, ScalarValue};

/// An error in the application.
#[derive(Debug, PartialEq, Clone)]
pub(crate) struct AppError {
    message: String,
    data: juniper::Value,
}

impl<T: Display> From<T> for AppError {
    fn from(e: T) -> AppError {
        AppError {
            message: format!("{}", e),
            data: juniper::Value::null(),
        }
    }
}

impl<S: ScalarValue> IntoFieldError<S> for AppError {
    fn into_field_error(self) -> FieldError<S> {
        FieldError::new(self.message, self.data).map_scalar_value()
    }
}

/// An error in the application.
impl AppError {
    /// Creates a new error with the given message and data.
    pub(crate) fn new(message: String, data: juniper::Value) -> Self {
        Self { message, data }
    }

    /// Creates a new error based on a validation error. This would best be replaced by some Serde integration for GraphQL values.
    pub(crate) fn from_validation(validation_errors: validator::ValidationErrors) -> Self {
        let keys = validation_errors.errors().keys();
        let mut errors = Vec::with_capacity(keys.len());
        for (key, error_kind) in validation_errors.errors() {
            // only include field validation errors
            if let validator::ValidationErrorsKind::Field(field_errors) = error_kind {
                // field validation error entries
                let entries = field_errors
                    .iter()
                    .map(|error| {
                        // add the code, message and params
                        let mut data = juniper::Object::with_capacity(3);
                        data.add_field("code", graphql_value!(error.code.to_string()));
                        data.add_field(
                            "message",
                            graphql_value!(error.message.as_ref().map(|s| s.to_string())),
                        );
                        data.add_field("params", graphql_value!(format!("{:?}", error.params)));
                        graphql_value!(data)
                    })
                    .collect::<Vec<juniper::Value>>();

                // field error data
                let mut error_data = juniper::Object::with_capacity(2);
                error_data.add_field("field", graphql_value!(*key));
                error_data.add_field("errors", juniper::Value::List(entries));
                errors.push(graphql_value!(error_data));
            }
        }

        Self {
            message: format!(
                "operation failed with validation errors on fields: {}",
                keys.cloned().collect::<Vec<&str>>().join(", ")
            ),
            data: juniper::Value::List(errors),
        }
    }
}
