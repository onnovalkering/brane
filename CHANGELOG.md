# Changelog

All notable changes to the Brane framework will be documented in this file.

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
