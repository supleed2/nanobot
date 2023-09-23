CREATE TABLE IF NOT EXISTS "pending" (
	"discord_id" BIGINT NOT NULL PRIMARY KEY,
	"shortcode" VARCHAR(16) NOT NULL,
	"realname" TEXT NOT NULL
)