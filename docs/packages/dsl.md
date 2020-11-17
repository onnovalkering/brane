---
layout: default
title: DSL
parent: Packages
nav_order: 2
---

# DSL scripts
A package can be built based on a [Bakery](/brane/bakery) script.

```shell
$ brane build script.bk
```

Point the `build` command of the Brane CLI to a file containing a Bakery script. The complete Bakery script will be converted into a package with a single function.

### Metadata
Relevant metadata can be added to the CWL document

| Field       | Required | Description                                 | 
|:------------|:---------|:--------------------------------------------|
| `name`      | Yes      | Will be used as the name of the package.    |
| `version`   | Yes      | Will be used as the version of the package. |

These fields are added to the script by using a header comment, at the start of the file:

```yaml
---
name: mypackage
version: 1.0.0
---

// ... script here
```

