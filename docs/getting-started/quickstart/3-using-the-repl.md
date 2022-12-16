# 3. Using the REPL

We can combine our two functions using Brane's DSL: [BraneScript](../../programming/branescript.md). Start by opening a new REPL:

```
$ brane repl
```

Execute the following statements, line-by-line, to retreive the README.md file and decode it (Fig. 1):

```go
import github;
import base64;

let owner := "onnovalkering";
let repo := "brane";

let readme := getreadme(owner, repo);
decode(readme.content);
```

{% hint style="danger" %}
BraneScript requires statements to be terminated with a semicolon (`;`).&#x20;
{% endhint %}

![Figure 1: executing package's functions from the REPL.](<../../.gitbook/assets/Screen Shot 2021-05-03 at 15.23.35.png>)

{% hint style="info" %}
Like the `test` command, adding the `--debug` option when starting the REPL results in more detailed (error) messages: `brane --debug repl`.
{% endhint %}
