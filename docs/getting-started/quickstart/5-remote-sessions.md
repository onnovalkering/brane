# 5. Remote sessions

With our packages published, we can run the script from [step 3](3-using-the-repl.md) as part of our Brane instance. Open a new remote REPL by specifying the address of the Brane instance:

```
$ brane repl --remote http://localhost:50053
```

A remote REPL sessions works similar as a local REPL, expect the registry of the Brane instance is used to determine which packages are available, also the functions are executed as part of the Brane instance.

![Figure 1: a remote REPL session works the same as a local REPL session.](<../../.gitbook/assets/Screen Shot 2021-05-03 at 16.18.30.png>)

Similarly, we can run the same script from a JupyterLab notebook (Use a **BraneScript** notebook):

![Figure 2: a JupyterLab notebook functions as a remote REPL session.](<../../.gitbook/assets/Screen Shot 2021-05-03 at 22.31.13.png>)

{% hint style="info" %}
Choose for a BraneScript kernel from the JupyterLab launcher.
{% endhint %}
