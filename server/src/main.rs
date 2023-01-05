#![deny(rustdoc::broken_intra_doc_links)]
#![warn(missing_docs)]

//! An inventory tracking web application.

#[macro_use]
extern crate derive_more;

mod batcher;
mod db;
mod graphql;
mod model;
mod store;

use std::env;
use std::sync::Arc;

use actix_web::{http, middleware, web, App, Error, HttpRequest, HttpResponse, HttpServer};
use async_graphql::http::GraphQLPlaygroundConfig;
use async_graphql_actix_web::{GraphQLRequest, GraphQLResponse, GraphQLSubscription};

use crate::graphql::{AppContext, AppSchema, Clients};

/// The route for the GraphQL playground.
async fn playground_route() -> Result<HttpResponse, Error> {
    let source = async_graphql::http::playground_source(
        GraphQLPlaygroundConfig::new("/graphql").subscription_endpoint("/subscriptions"),
    );
    Ok(HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(source))
}

/// The route for the GraphQL endpoint.
async fn graphql_route(req: GraphQLRequest, schema: web::Data<AppSchema>) -> GraphQLResponse {
    schema.execute(req.into_inner()).await.into()
}

/// The route for the GraphQL subscriptions.
async fn subscription_route(
    req: HttpRequest,
    payload: web::Payload,
    schema: web::Data<AppSchema>,
) -> Result<HttpResponse, Error> {
    GraphQLSubscription::new(async_graphql::Schema::clone(&*schema)).start(&req, payload)
}

/// Gets the context for the application.
async fn get_context() -> AppContext {
    // create the redis client and db pool, storing them in the context
    let redis = Arc::new(
        store::get_client()
            .await
            .expect("unable to connect to redis"),
    );
    let postgres = Arc::new(db::get_pool().await);

    let clients = Clients { postgres, redis };

    let mut loaders = anymap2::Map::new();
    batcher::register_loaders(&clients, &mut loaders);

    AppContext {
        clients,
        loaders: Arc::new(loaders),
    }
}

/// Entrypoint for the actix web application.
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let context = get_context().await;
    let schema = graphql::schema_builder().data(context).finish();

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(schema.clone()))
            .wrap(middleware::Compress::default())
            .wrap(middleware::Logger::default())
            .wrap(
                actix_cors::Cors::default()
                    .allow_any_origin()
                    .allowed_methods(vec!["POST", "GET"])
                    .allowed_headers(vec![http::header::ACCEPT, http::header::CONTENT_TYPE])
                    .max_age(3600),
            )
            .service(
                web::resource("/graphql")
                    .route(web::post().to(graphql_route))
                    .route(web::get().to(graphql_route)),
            )
            .service(web::resource("/subscriptions").route(web::get().to(subscription_route)))
            .service(web::resource("/playground").route(web::get().to(playground_route)))
            .default_service(web::route().to(HttpResponse::NotFound))
    })
    .bind(format!(
        "{}:{}",
        env::var("ACTIX_ADDRESS").expect("ACTIX_ADDRESS must be set"),
        env::var("PORT").expect("PORT must be set")
    ))?
    .run()
    .await
}

/// Integration level tests.
#[cfg(test)]
mod test {
    use super::*;

    use actix_web::test;

    /// Macro to set up the test server.
    macro_rules! test_server {
        () => {{
            let context = get_context().await;
            let schema = graphql::schema_builder().data(context).finish();
            test::init_service(
                App::new()
                    .app_data(web::Data::new(schema.clone()))
                    .service(web::resource("/graphql").route(web::post().to(graphql_route))),
            )
            .await
        }};
    }

    #[actix_rt::test]
    async fn test_empty_item_name() {
        let mut app = test_server!();
        let req = test::TestRequest::post()
            .uri("/graphql")
            .insert_header(http::header::ContentType::json())
            .set_json(serde_json::json!({
                "query": r#"mutation { createItem(item: { name: "" }) { id } }"#
            }))
            .to_request();
        let resp: serde_json::value::Value = test::call_and_read_body_json(&mut app, req).await;
        assert!(!resp["errors"][0]["message"].is_null());
    }

    #[actix_rt::test]
    async fn test_duplicate_item_sku() {
        let mut app = test_server!();
        let mut resp: serde_json::value::Value = serde_json::json!({});
        for _ in 0..2 {
            let req = test::TestRequest::post()
                .uri("/graphql")
                .insert_header(http::header::ContentType::json())
                .set_json(serde_json::json!({
                    "query": r#"mutation { createItem(item: { name: "name", sku: "ABC" }) { id } }"#
                }))
                .to_request();
            resp = test::call_and_read_body_json(&mut app, req).await;
        }
        assert!(!resp["errors"][0]["message"].is_null());
    }

    #[actix_rt::test]
    async fn test_nonexistent_transaction_item() {
        let mut app = test_server!();
        let req = test::TestRequest::post().uri("/graphql").insert_header(http::header::ContentType::json()).set_json(serde_json::json!({
            "query": r#"mutation { createTransaction(transaction: { itemId: 0, quantity: 10 }) { id } }"#
        })).to_request();
        let resp: serde_json::value::Value = test::call_and_read_body_json(&mut app, req).await;
        assert!(!resp["errors"][0]["message"].is_null());
    }

    #[actix_rt::test]
    async fn test_nonexistent_transaction_location() {
        let mut app = test_server!();
        // create a test item
        let req = test::TestRequest::post()
            .uri("/graphql")
            .insert_header(http::header::ContentType::json())
            .set_json(serde_json::json!({
                "query": r#"mutation { createItem(item: { name: "TestItem" }) { id } }"#
            }))
            .to_request();
        let resp: serde_json::value::Value = test::call_and_read_body_json(&mut app, req).await;
        let id = resp["data"]["createItem"]["id"].as_i64().unwrap();

        let req = test::TestRequest::post()
            .uri("/graphql")
            .insert_header(http::header::ContentType::json())
            .set_json(serde_json::json!({
                "query":
                    format!(
                        "{}{}{}",
                        r#"mutation { createTransaction(transaction: { itemId: "#,
                        id,
                        r#", locationId: 0, quantity: 10 }) { id } }"#
                    )
            }))
            .to_request();
        let resp: serde_json::value::Value = test::call_and_read_body_json(&mut app, req).await;
        assert!(!resp["errors"][0]["message"].is_null());
    }

    #[actix_rt::test]
    async fn test_delete_location_nulls_and_delete_item_deletes_transaction() {
        let mut app = test_server!();
        // create a test item
        let req = test::TestRequest::post()
            .uri("/graphql")
            .insert_header(http::header::ContentType::json())
            .set_json(serde_json::json!({
                "query": r#"mutation { createItem(item: { name: "TestItem" }) { id } }"#
            }))
            .to_request();
        let resp: serde_json::value::Value = test::call_and_read_body_json(&mut app, req).await;
        let item_id = resp["data"]["createItem"]["id"].as_i64().unwrap();

        // create a test location
        let req = test::TestRequest::post()
            .uri("/graphql")
            .insert_header(http::header::ContentType::json())
            .set_json(serde_json::json!({
                "query": r#"mutation { createLocation(location: { name: "Toronto" }) { id } }"#
            }))
            .to_request();
        let resp: serde_json::value::Value = test::call_and_read_body_json(&mut app, req).await;
        let location_id = resp["data"]["createLocation"]["id"].as_i64().unwrap();

        // create a test transaction
        let req = test::TestRequest::post()
            .uri("/graphql")
            .insert_header(http::header::ContentType::json())
            .set_json(serde_json::json!({
                "query":
                    format!(
                        "{}{}{}",
                        r#"mutation { createTransaction(transaction: { itemId: "#,
                        item_id,
                        r#", quantity: 10 }) { id } }"#
                    )
            }))
            .to_request();
        let resp: serde_json::value::Value = test::call_and_read_body_json(&mut app, req).await;
        let transaction_id = resp["data"]["createTransaction"]["id"].as_i64().unwrap();

        // delete the test location
        let req = test::TestRequest::post()
            .uri("/graphql")
            .insert_header(http::header::ContentType::json())
            .set_json(serde_json::json!({
                "query":
                    format!(
                        "{}{}{}",
                        r#"mutation { deleteLocation(id: "#, location_id, r#" ) { id } }"#
                    )
            }))
            .to_request();
        let resp: serde_json::value::Value = test::call_and_read_body_json(&mut app, req).await;
        assert_eq!(
            resp["data"]["deleteLocation"]["id"].as_i64().unwrap(),
            location_id
        );

        // check that the test transaction location is null
        let req = test::TestRequest::post()
            .uri("/graphql")
            .insert_header(http::header::ContentType::json())
            .set_json(serde_json::json!({
                "query":
                    format!(
                        "{}{}{}",
                        r#"{ transaction(id: "#, transaction_id, r#") { location { id } } }"#
                    )
            }))
            .to_request();
        let resp: serde_json::value::Value = test::call_and_read_body_json(&mut app, req).await;
        assert_eq!(
            resp["data"]["transaction"]["location"],
            serde_json::json!(null)
        );

        // delete the test item
        let req = test::TestRequest::post()
            .uri("/graphql")
            .insert_header(http::header::ContentType::json())
            .set_json(serde_json::json!({
                "query":
                    format!(
                        "{}{}{}",
                        r#"mutation { deleteItem(id: "#, item_id, r#" ) { id } }"#
                    )
            }))
            .to_request();
        let resp: serde_json::value::Value = test::call_and_read_body_json(&mut app, req).await;
        assert_eq!(resp["data"]["deleteItem"]["id"].as_i64().unwrap(), item_id);

        // check that the test transaction was deleted
        let req = test::TestRequest::post()
            .uri("/graphql")
            .insert_header(http::header::ContentType::json())
            .set_json(serde_json::json!({
                "query":
                    format!(
                        "{}{}{}",
                        r#"{ transaction(id: "#, transaction_id, r#") { id } }"#
                    )
            }))
            .to_request();
        let resp: serde_json::value::Value = test::call_and_read_body_json(&mut app, req).await;
        assert_eq!(resp["errors"][0]["message"].as_str().unwrap(), "not found");
    }
}
