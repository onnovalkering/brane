# Changelog

All notable changes to the Brane framework will be documented in this file.

## [0.4.0] - 2021-08-11
### Added
- BraneScript, an alternative to Bakery with more a C-like syntax.
- GraphQL endpoint for querying application event logs, including subscriptions.
- Initial support for proxies and bridge functions: `brane-net`.
- Allow checkout folder name to be different than 'brane' (by [romnn](https://github.com/romnn)).
- Automated (daily) audits and multi-platform builds using GitHub actions.
- Optional flag to keep temporary package build files.
- Automatically add `token` and `server` arguments for OAS functions. 

## Changed
- Use seperate service for scheduling functions: `brane-job`.
- Use seperate library for OpenAPI support: `brane-oas`.
- REPL is now based on the `rustyline` library.
- Use gRPC for drivers (REPL and Jupyter kernel).
- Switched from Cassandra to ScyllaDB, and removed PostgreSQL dependency.
- DSL implementation is based on parser combinatorics, with `nom`.
- Switched from `actix` to `warp` as the framework for `brane-api`.

## Fixed
- Minor fixes for the word count quickstart.
- Correctly convert between DSL values and specification values.

## [0.3.0] - 2021-03-03
### Added
- Generate convenience function for CWL workflows with a single required parameter.
- `run` command to run DSL script from files. 
- `import` command to import packages from a GitHub repository.
- JupyterLab-based registry viewer.

## Changed
- The `import` DSL statement accepts multiple packages on the same line.
- Optional properties do not have to be specified while creating an object in the DSL.
- Cell output shows progress indicator and has time statistics.

## [0.2.0] - 2020-12-15
### Added
- Contributing guide, code of conduct, and issue templates (bug & feature).
- LOFAR demonstration
- Session attach/detach mechanism in JupyterLab.
- Custom renderers in JupyterLab.

### Changed
- Docker, HPC (Xenon), and Kubernetes runners are now configurable.
- Removing a package also removes it locally from Docker.
- CWL packages are now also locally testable.

### Fixed
- Various bug fixes and improvements.
- Allow pointers when creating arrays and objects in Bakery.

## [0.1.0] - 2020-06-04
### Added
- Initial implementation.
