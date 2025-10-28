create table if not exists "pending" (
	"discord_id" bigint not null primary key,
	"shortcode" varchar(16) not null,
	"realname" text not null
)