CREATE TABLE `packages` (
  `id` INTEGER NOT NULL PRIMARY KEY,
  `uuid` VARCHAR NOT NULL,
  `kind` VARCHAR NOT NULL,
  `name` VARCHAR NOT NULL,
  `version` VARCHAR NOT NULL,
  `created` DATETIME NOT NULL,
  `description` VARCHAR,
  `functions_json` VARCHAR,
  `types_json` VARCHAR
);
