create table if not exists users
(
    id                  uuid primary key default gen_random_uuid(),
    email               text unique not null,
    username            text unique not null,
    password_hash       text        not null,
    registration_date   timestamptz not null default now()
);

