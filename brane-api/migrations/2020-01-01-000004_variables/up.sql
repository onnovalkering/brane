CREATE TABLE "variables" (
    "id" SERIAL PRIMARY KEY
  , "session" INTEGER NOT NULL REFERENCES "sessions"("id") 

  -- metadata
  , "created" TIMESTAMP NOT NULL
  , "updated" TIMESTAMP

  -- content
  , "name" VARCHAR NOT NULL
  , "type" VARCHAR NOT NULL
  , "content_json" TEXT
);

CREATE UNIQUE INDEX "variables_name" ON "variables"("name", "session");
