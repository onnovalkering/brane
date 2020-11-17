---
layout: default
title: CWL
parent: Packages
nav_order: 1
---

# Common Workflow Language
A package can be built based on a [CWL worklow](https://www.commonwl.org/v1.2/Workflow.html) or [CWL command line tool](https://www.commonwl.org/v1.2/CommandLineTool.html).

```shell
$ brane build document.cwl
```

Point the `build` command of the Brane CLI to a CWL document. The CWL builder will automatically gather and include all the related files, as referenced from the CWL document.
Only files with a single workflow or command line tool can be used. Thus is converted into a package with a single function.

### Metadata
Relevant metadata can be added to the CWL document

| Field       | Required | Description                                 | 
|:------------|:---------|:--------------------------------------------|
| `s:name`    | Yes      | Will be used as the name of the package.    |
| `s:version` | Yes      | Will be used as the version of the package. |
| `label`     | Yes      | Will be used as the function's name.        |

Here `s:` is a namespace that corresponds to `schema.org`:

```yaml
$namespaces:
  s: http://schema.org/
```

