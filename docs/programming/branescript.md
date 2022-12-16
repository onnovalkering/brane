---
description: A C-like programming language with advanced multi-site constructs.
---

# BraneScript

More elaborate documentation is coming soon.

## Syntax

### Comments

BraneScript supports single-line comments:

```go
// This is a single-line comment.
```

### Imports

Packages can be imported with the `import` keyword:

```go
import arithmetic;
```

### Variable

Variables are created with the `let` keyword:

```go
let my_variable := "Hello, world";
```

Once declared, variables can be updated using assignment:

```go
my_variable := my_variable + "!";
```

### Arrays

Arrays can be created as follows:

```
let my_array := [1, 2, 3];
```

### Functions

Functions can be created using the `func` keyword:

```go
func sum(lhs, rhs) {
    return lhs + rhs;
}
```

The function call be called as follows:

```go
sum(1, 2);
```

### Conditionals

If-statements are supported as well:

```
if (true) {
    return "TRUE!";
} else {
    return "FALSE!";
}
```

### Loops

For-loops are supported as well:

```go
let sum := 0;

for (let i := 0; i < 10; i := i + 1) {
    sum := sum + i;
}

return sum;
```
