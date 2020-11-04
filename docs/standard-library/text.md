---
layout: default
title: Text
parent: Standard library
nav_order: 2
---

# Text
This package contains functions for text manipulation.

```go
import "text"
```

### concat
Concatenate a string with another string or integer into a new string.

```go
myval := "Hello, " + "world!"
myval := "The answer is " + 42 + "."
```

### split
Splits a string on whitespace, returns an array of its segments.

```go
mywords := "This is a sentence."
```