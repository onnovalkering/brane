CREATE TABLE "invocations" (
    "id" SERIAL PRIMARY KEY
  , "session" INTEGER NOT NULL REFERENCES "sessions"("id")

  -- metadata
  , "created" TIMESTAMP NOT NULL
  , "name" VARCHAR
  , "started" TIMESTAMP
  , "stopped" TIMESTAMP  
  , "uuid" VARCHAR NOT NULL

  -- content
  , "instructions_json" TEXT NOT NULL
  , "status" VARCHAR NOT NULL
);

CREATE UNIQUE INDEX "invocation_uuid" ON invocations("uuid");
