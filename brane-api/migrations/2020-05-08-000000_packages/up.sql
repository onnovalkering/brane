CREATE TABLE `packages` (
  `id` INTEGER NOT NULL PRIMARY KEY

  -- metadata
  ,`created` DATETIME NOT NULL
  ,`kind` VARCHAR NOT NULL
  ,`name` VARCHAR NOT NULL
  ,`uploaded` DATETIME NOT NULL
  ,`uuid` VARCHAR NOT NULL
  ,`version` VARCHAR NOT NULL

  -- content
  ,`description` VARCHAR
  ,`functions_json` VARCHAR
  ,`types_json` VARCHAR

  -- file
  ,`checksum` BIGINT NOT NULL
  ,`filename` VARCHAR NOT NULL
);

CREATE UNIQUE INDEX `package_version` ON packages(`name`, `version`);
