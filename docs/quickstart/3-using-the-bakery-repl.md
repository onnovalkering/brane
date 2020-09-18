---
layout: default
title: "3. Using the Bakery REPL"
nav_order: 3
parent: Quickstart
---

# 3. Using the Bakery REPL
<span class="label label-blue">APPLICATION</span>

... intro

## REPL...


## Combine download and decoding
We can create hierachical packages by writing script in the [Bakery](/brane/references/bakery) language. In this example, we combine downloading of the README and the subseqent Base64 decoding into a single function:



## __[Bonus]__ Create a DSL package
We can create a package based on this... specify name and version in header of script

```go
---
name: getreadme
version: 1.0.0
---
import "github"
import "base64"

owner := ?? as String
repo := ?? as String

readme := getreadme owner repo
return b64decode readme.content
```


Assuming the DSL script is saved as `readme.bk`, the package can be build using:
```shell
$ brane build readme.bk
```

[Previous](/brane/quickstart/2-decode-from-base64.html){: .btn .btn-outline }
[Next](/brane/quickstart/4-publishing-packages.html){: .btn .btn-outline }