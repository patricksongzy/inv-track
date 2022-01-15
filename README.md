# inv-track
a simple inventory tracking web application

## Running
**note** docker compose v3 is required

for development: `docker compose up`\
integration tests: `docker compose -f docker-compose.yml -f integration-test.yml up`\
for production: `docker compose -f docker-compose.yml -f production.yml up`

## Technologies Used
* Rust + actix-web + juniper + sqlx
* Postgresql + Redis
* Docker Compose

## GraphQL
* chose GraphQL of the amount of querying done (also because it's interesting)
* API endpoint is `/graphql`, playground IDE is `/playground`
* source code in `/server/src/graphql`
## Batching Dataloaders (N+1 Problem)
* batching dataloaders to mitigate the N+1 Problem
* source code in `/server/src/batcher`
## Subscriptions
* subscriptions for real-time data updates
* endpoint is `/subscriptions`
* source code in `/server/src/graphql/subscription.rs`

## Tetsting
* tests are located in `/server/src/main.rs` and `/server/src/batcher/id_loader.rs`

## Extensibility
* add Redis caching layer
  * currently Redis is only used for the pubsub pattern
* use SeaORM instead of just SQLx
  * decided on SQLx because I wanted to write a bit of SQL code instead of learning an ORM's query language
* add multi stage docker builds
  * will trim down on image sizes
* add pagination
* we explicitly choose to allow for negative quantities
  * in the future, warnings can be added to the response
* better error handling
  * fix errors so that data is still returned for the other queries

### No NginX
* ideally we would be using NginX, but since we're not as focused on deployment, the current setup works

## Third-party Crates
multiple open-source crates were used to build this. Their licences are included below.

| Crate       | Licence                                                           |
|-------------|-------------------------------------------------------------------|
| validator   | [MIT](https://github.com/Keats/validator/blob/master/LICENSE)     |
