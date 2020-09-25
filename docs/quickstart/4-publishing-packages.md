---
layout: default
title: "4. Publishing packages"
nav_order: 4
parent: Quickstart
---

# 4. Publishing packages
<span class="label label-blue">APPLICATION</span> <span class="label label-red">SYSTEM</span>

Before the domain scientist can write the final application, i.e. word count, all the created packages have to be published. This is done by pushing (uploading) them to the runtime system's registry.

First we have to pair the CLI to the backend. Point the `login` command to your backend installation:

```shell
$ brane login 'http://localhost:8080' --username joyvan
```

After logging in, we can push the packages (Fig. 1):
```shell
$ brane push "github" 1.0.0
$ brane push "base64" 1.0.0
$ brane push "getreadme" 1.0.0
```

<p style="text-align: center">
    <img src="/brane/assets/img/brane-push.png" style="margin-bottom: -35px" alt="pushing packages">
    <br/>
    <sup>Figure 1: pushing packages</sup>
</p>

<!-- TODO: add remove -->

[Previous](/brane/quickstart/3-using-the-bakery-repl.html){: .btn .btn-outline }
[Next](/brane/quickstart/5-using-bakery-notebooks.html){: .btn .btn-outline }