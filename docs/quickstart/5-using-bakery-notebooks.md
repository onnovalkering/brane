---
layout: default
title: "5. Using Bakery notebooks"
nav_order: 5
parent: Quickstart
---

# 5. Using Bakery notebooks
<span class="label label-green">USERS</span>

- final workflow: REPL + word count..

Now we can start writing workflows. Consider the following workflow:
```go
import "cowsay"
import "getreadme"
import "text"

readme := getreadme "onnovalkering" "brane"
words := split readme

message := "This readme contains " + words.length + " words."
cowsay message
```

Writing this workflow is very easy and we don't have to deal with any technicalities, under the hood Brane will take care of:

- calling a remote web service (`getreadme`);
- executing containerized code (`b64decode`);
- executing a CWL workflow (`cowsay`).
- basic string manipulation (`split`; from the [std](/brane/references/standard-library.html))

![test](/brane/assets/img/jupyter.png)

## __Bonus__: notebook magic


[Previous](/brane/quickstart/4-publishing-packages.html){: .btn .btn-outline }