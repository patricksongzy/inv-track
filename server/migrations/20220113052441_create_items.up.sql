create table items (
    id serial primary key,
    sku text unique,
    name text not null,
    supplier text,
    description text
);
