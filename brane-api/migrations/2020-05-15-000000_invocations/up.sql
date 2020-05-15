CREATE TABLE "invocations" (
    "id" SERIAL PRIMARY KEY

  -- metadata
  , "created" TIMESTAMP NOT NULL
  , "name" VARCHAR
  , "uuid" VARCHAR NOT NULL

  -- content
  , "status" VARCHAR NOT NULL
  , "arguments_json" TEXT NOT NULL
  , "instructions_json" TEXT NOT NULL
);

CREATE UNIQUE INDEX "invocation_uuid" ON invocations("uuid");
