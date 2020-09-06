CREATE TABLE "packages" (
    "id" SERIAL PRIMARY KEY

  -- metadata
  , "created" TIMESTAMP NOT NULL
  , "kind" VARCHAR NOT NULL
  , "name" VARCHAR NOT NULL
  , "uploaded" TIMESTAMP NOT NULL
  , "uuid" VARCHAR NOT NULL
  , "version" VARCHAR NOT NULL

  -- content
  , "description" VARCHAR
  , "functions_json" TEXT
  , "source" TEXT
  , "types_json" TEXT

  -- file
  , "checksum" BIGINT NOT NULL
  , "filename" VARCHAR NOT NULL
);

CREATE UNIQUE INDEX "package_uuid" ON packages("uuid");
CREATE UNIQUE INDEX "package_version" ON packages("name", "version");
