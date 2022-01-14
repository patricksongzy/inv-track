create table transactions(
    id serial primary key,
    item_id integer not null,
    location_id integer null,
    transaction_date timestamptz,
    quantity integer not null,
    comment text,
    foreign key (item_id) references items on delete cascade,
    foreign key (location_id) references locations on delete set null
);
