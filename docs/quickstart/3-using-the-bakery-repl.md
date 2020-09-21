---
layout: default
title: "3. Using the Bakery REPL"
nav_order: 3
parent: Quickstart
---

# 3. Using the Bakery REPL
<span class="label label-blue">APPLICATION</span>

Up until now, we've create two seperate packages: `github` and `base64`. Let's try to use them together. 

We'll use [Bakery](/brane/bakery) for this, Brane's <abbr title="Domain-specific language">DSL</abbr> for developing applications. Since we'll only use packages that are locally available, we can use the Bakery <abbr title="Read-eval-print loop">REPL</abbr>. It's a convenient prototyping and development tool:

```shell
$ brane repl
```

Enter the following statements, one by one, in the REPL shell. It will dowload the README.md file from a GitHub repository using `github` package, and decode its content using the `base64` package (Fig. 1):

```go
brane> import "github"
brane> readme := getreadme "onnovalkering" "brane"
brane> import "base64"
brane> readme.content decoded
```

<p style="text-align: center">
    <img src="/brane/assets/img/brane-repl.png" style="margin-bottom: -35px" alt="using the Brane REPL">
    <br/>
    <sup>Figure 1: using the Brane REPL</sup>
</p>

Remember the (postfix) call pattern that we added to the `container.yml` in the previous step? Because of this pattern, we can call the decode function as we did: ```<argument> decoded```. This pre-/in-/postfix mechanism allows us to create sentence-like statements. For more details see the [Bakery](/brane/bakery) page.

## Building a DSL package
Because we always have to decode a README.md file from Base64, it is handy to create a wrapper function that does this for us. We do this by writing a Bakery script, and turn it into a function later:
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
return readme.content decoded
```
We specify the name and the version in the header. We bring the packages that we want to use into scope with the `import` keyword. Input parameters are specified by creating `??` (unkown) variables. Bakery variables need to have an associated type. Therefore, we indicate how to treat the unkowns using the `as` keyword. At runtime, these unkowns will be replaced with the specific arguments. Then, we execute the functions, like before. The `return` keyword is to mark the output, its type is inferred.

We can build a package based on this script, and start using it as a function, using the <abbr title="Command-line interface">CLI</abbr>:
```shell
$ brane build readme.bk
```

Our new `getreadme` function can be used just as any other function (Fig. 2):
```shell
$ brane test getreadme
```

<p style="text-align: center">
    <img src="/brane/assets/img/brane-test-getreadme.png" style="margin-bottom: -35px" alt="package tester output">
    <br/>
    <sup>Figure 2: package tester output</sup>
</p>

[Previous](/brane/quickstart/2-decode-from-base64.html){: .btn .btn-outline }
[Next](/brane/quickstart/4-publishing-packages.html){: .btn .btn-outline }