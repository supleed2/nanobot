CREATE TABLE IF NOT EXISTS "manual" (
	"discord_id" BIGINT NOT NULL PRIMARY KEY,
	"shortcode" VARCHAR(16) NOT NULL,
	"nickname" TEXT NOT NULL,
	"realname" TEXT NOT NULL,
	"fresher" BOOLEAN NOT NULL
)