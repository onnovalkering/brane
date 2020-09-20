---
layout: default
title: "4. Publishing packages"
nav_order: 4
parent: Quickstart
---

# 4. Publishing packages
<span class="label label-blue">APPLICATION</span> <span class="label label-red">SYSTEM</span>

We can publish packages by pushing them to the registry of the backend deployment. But first we have to pair the CLI to the backend. Point the `login` command to the API of your backend installation:

```shell
$ brane login 'http://localhost:8080' --username joyvan
```

After logging in, we can push the packages:
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