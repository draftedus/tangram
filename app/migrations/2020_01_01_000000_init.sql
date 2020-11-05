create table users (
	id char(32) primary key,
	created_at bigint not null,
	email varchar(320) unique not null
);

create table organizations (
	id char(32) primary key,
	created_at bigint not null,
	name text
);

create table organizations_users (
	organization_id char(32) references organizations (id) on delete cascade not null,
	user_id char(32) references users (id) not null,
	is_admin bool not null,
	primary key (organization_id, user_id)
);

create table codes (
	id char(32) primary key,
	created_at bigint not null,
	deleted_at bigint,
	user_id char(32) references users (id) not null,
	code char(6) not null
);

create table repos (
	id char(32) primary key,
	created_at bigint not null,
	title varchar(64) not null,
	organization_id char(32) references organizations (id) on delete cascade,
	user_id char(32) references users (id) on delete cascade,
	/* ensure that organization_id and user_id are not both set */
	constraint single_owner check (
		(organization_id is null and user_id is null)
		or
		(organization_id is null and user_id is not null)
		or
		(organization_id is not null and user_id is null)
	)
);

create table models (
	id char(32) primary key,
	repo_id char(32) references repos (id) on delete cascade not null,
	created_at bigint not null,
	data text not null
);

create index code_index on codes (code);

create table tokens (
	id char(32) primary key,
	created_at bigint not null,
	deleted_at bigint,
	title text,
	token char(32) unique not null,
	user_id char(32) references users (id) not null
);

create table predictions (
	id char(32) primary key,
	model_id char(32) references models (id) on delete cascade not null,
	date bigint not null,
	created_at bigint not null,
	identifier varchar(64) not null,
	input text not null,
	output text not null
);

create table true_values (
	id char(32) primary key,
	model_id char(32) references models (id) on delete cascade not null,
	date bigint not null,
	created_at bigint not null,
	identifier varchar(64) not null,
	value text not null
);

create table production_stats (
	model_id char(32) references models (id) on delete cascade not null,
	hour bigint not null,
	data text not null,
	primary key (model_id, hour)
);

create table production_metrics (
	model_id char(32) references models (id) on delete cascade not null,
	hour bigint not null,
	data text not null,
	primary key (model_id, hour)
);
