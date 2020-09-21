---
layout: default
title: "5. Using Bakery notebooks"
nav_order: 5
parent: Quickstart
---

# 5. Using Bakery notebooks
<span class="label label-green">USERS</span>

Now we've published our package to the Brane backend, we can write the final application using JupyterLab (Fig. 1), part of the backend installation (defaults to port `8888`, e.g. [http://localhost:8888](http://localhost:8888)).


```go
import "getreadme"
import "text"

readme := getreadme "onnovalkering" "brane"
words := split readme

message := "This README.md has " + words.length + " words. Threshold met? "

if words.length > 100: message := message + " Yes."
else: message := message + " No."

return message
```
We use the `getreadme` package that we created in the previous step. This package in turn makes use of the `github` and `base64` packages. The `split` function is provided by the `text` package, part of the [standard library](/brane/standard-library/standard-library.html). This function splits a `string` based on whitespace, returning an `string[]` array. On this array is a `length` property that contains the number of array entries, i.e. the number of words in the README.md file. By checking if this meets a certain threshold a certian message is displayed.


<p style="text-align: center">
    <img src="/brane/assets/img/wordcount.png" style="margin-bottom: -25px" alt="word count application in JupyterLab">
    <br/>
    <sup>Figure 1: word count application in JupyterLab</sup>
</p>

__TIP__: the backend's JupyterLab includes a registry browser, listing all available functions.

[Previous](/brane/quickstart/4-publishing-packages.html){: .btn .btn-outline }