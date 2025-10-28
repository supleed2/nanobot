create table if not exists "members" (
	"discord_id" bigint not null primary key,
	"shortcode" varchar(16) not null constraint "users_shortcode_unique" unique,
	"nickname" text not null,
	"realname" text not null,
	"fresher" varchar(16) not null,
	check ("fresher" in ('no', 'yes_pg', 'yes_ug'))
)