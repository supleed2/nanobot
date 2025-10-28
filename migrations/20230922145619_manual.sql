create table if not exists "manual" (
	"discord_id" bigint not null primary key,
	"shortcode" varchar(16) not null,
	"nickname" text not null,
	"realname" text not null,
	"fresher" varchar(16) not null,
	check ("fresher" in ('no', 'yes_pg', 'yes_ug'))
)