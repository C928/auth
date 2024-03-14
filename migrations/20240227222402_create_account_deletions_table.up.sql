create table if not exists account_deletions
(
    id                  varchar(150) primary key,
    account_id          uuid not null references users(id),
    registration_date   timestamptz not null default now()
);

alter table users add column requested_deletion boolean default null;