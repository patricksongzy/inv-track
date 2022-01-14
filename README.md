# inv-track
a simple inventory tracking web application

## Running
**note** docker compose v3 is required

for development: `docker compose up`\
integration tests: `docker compose -f docker-compose.yml -f integration-test.yml up`\
for production: `docker compose -f docker-compose.yml -f production.yml up`

### Cleanup
`Ctrl+C` then `docker compose down` and prune associated volumes, images and networks

## Design
### Technologies Used
* Rust + actix-web + juniper + sqlx
* Postgresql + Redis
* Docker Compose

### GraphQL
* batching dataloaders to mitigate the N+1 Problem

### Extensibility
* Redis caching layer
  * currently Redis is only used for the pubsub pattern
* use SeaORM instead of just SQLx
  * decided on SQLx because I wanted to write a bit of SQL code instead of learning an ORM's query language
* multi stage docker builds
  * will trim down on image sizes

### No NginX
* ideally we would be using NginX, but since we're not as focused on deployment, the current setup works

## Third-party Crates
multiple open-source crates were used to build this. Their licences are included below.

| Crate       | Licence                                                           |
|-------------|-------------------------------------------------------------------|
| actix-web   | [MIT/Apache 2.0]() |
| serde       | [MIT/Apache 2.0](https://github.com/serde-rs/serde#license)       |
| juniper     | [MIT/Apache 2.0]() |
| sqlx        | [MIT/Apache 2.0]() |
| validator   | [MIT](https://github.com/Keats/validator/blob/master/LICENSE)     |
