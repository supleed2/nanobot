CREATE TABLE IF NOT EXISTS "members" (
	"discord_id" BIGINT NOT NULL PRIMARY KEY,
	"shortcode" VARCHAR(16) NOT NULL CONSTRAINT "users_shortcode_unique" UNIQUE,
	"nickname" TEXT NOT NULL,
	"realname" TEXT NOT NULL,
	"fresher" BOOLEAN NOT NULL
)