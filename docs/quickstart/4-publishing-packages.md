---
layout: default
title: "4. Publishing packages"
nav_order: 4
parent: Quickstart
---

# 4. Publishing packages
<span class="label label-blue">APPLICATION</span> <span class="label label-red">SYSTEM</span>

We can publish packages by pushing them to the registry of a Brane instance. But first we have to pair the CLI to the brane instance. This is done by logging in (use the hostname of your instance):
```shell
$ brane login http://localhost:8080 --username joyvan
```

After logging in, we can push the packages:
```shell
$ brane push github 1.0.0
$ brane push base64 1.0.0
$ brane push getreadme 1.0.0
$ brane push cowsay 1.0.0
```

- screenshot (all four)

- in case we want to remove, use brane remove [--registry]


Now we can use these packages in our [workflows](/brane/guide/workflows).

[Previous](/brane/quickstart/3-using-the-bakery-repl.html){: .btn .btn-outline }
[Next](/brane/quickstart/5-using-bakery-notebooks.html){: .btn .btn-outline }