create table if not exists "gaijin" (
	"discord_id" bigint not null primary key,
	"name" text not null,
	"university" text not null
)