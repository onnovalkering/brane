CREATE TABLE `invocations` (
  `id` INTEGER NOT NULL PRIMARY KEY

  -- metadata
  ,`created` DATETIME NOT NULL
  ,`name` VARCHAR
  ,`uuid` VARCHAR NOT NULL

  -- content
  ,`status` VARCHAR NOT NULL
  ,`arguments_json` VARCHAR NOT NULL
  ,`instructions_json` VARCHAR NOT NULL
);

CREATE UNIQUE INDEX `invocation_uuid` ON packages(`uuid`);
