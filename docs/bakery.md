---
layout: default
title: Bakery
nav_order: 6
description: "bakery"
permalink: /bakery
---

# Bakery
On this page we cover all aspects of Bakery, including its [syntax](#syntax). <!-- and [notebook usage](#notebook-usage). -->

## Introduction
Bakery is a high-level <abbr title="Domain-specific language">DSL</abbr> for writing scientific applications, influenced by Cookery [[branowski2018]](https://doi.org/10.1109/eScience.2018.00102). It has been designed such that Bakery code is __intuitive to write__ and __easy to reason about__. This is beneficial for maintainability, but also makes Bakery accessible for users with limited programming experience. The two main features of Bakery are its __type safety__ and __sentence-like statements__.

### Type safety
All variables must have an associated type. Based on these types, the Bakery __type system__ is able to identify type errors early during development. For instance: passing text to a function when a number is expected. This prevents certain errors during runtime and improves the development productivity. 

Bakery has six __built-in types__, arrays, and supports custom types, i.e. [objects](#objects):

| Type      | Description                            | 
|:----------|:---------------------------------------|
| Boolean   | represents logical true or false.      |
| Directory | holds the location of a directory.     |
| File      | holds the location of a file.          |
| Integer   | number without a decimal part (64-bit) |
| Real      | number with a decimal part (64-bit)    |
| String    | unicode text (UTF-8)                   |

The `Boolean`, `Integer`, `Real`, and `String` types have their own representations:

```go
true  // boolean
false // boolean
10000 // Integer
10.00 // Real
"Sun" // String
```

The `Directory` and `File` types are created as [objects](#objects), where the file paths are URLs:

```go
new Directory { url: "file:///home/joyvan" }
new File { url: "file:///home/joyvan/test.txt" }
```

__Note__: local file paths are composed by prepending `file://` to the file path, as demonstrated above.

### Sentence-like statements
The statements in Bakery typically follow a sentence-like structure. This is possible due to a special syntax mechanism. With this mechanism, function authors specify with __pre-/in-/postfix patterns__ the notation of how their functions are called. Consider an arbitrary `write` function, that writes text to a file. Normally, we would write the function call as follows (assuming `text` and `file` as parameters):

```go
write(text, file)
```

For Bakery, we specify `write` as the prefix and `to` as an infix. Now we call the function as follows:

```go
write text to file
```

__Note__: patterns can be reused among functions, if the arguments have different types (overloading).

Multiple function calls can be combined into a single statement, consider the statement:

```go
transfer first 5 files to new_temp_directory
```

This statement is composed of three functions, a integer value `5`, and an array with files (`files`):

- __transfer__: transfers one or more files to a directory (`transfer <File[]> to <Directory>`)
- __first__: gets the first _x_ number of elements from an array (`first 5 <File[]>`)
- __create__: creates a new temporary directory (`new_temp_directory`)

## Syntax
In the next sections, we describe the Brane syntax per topic. 

### Comments
Both single-line and multi-line comments are supported:

```go
// This is a single-line comment

--- 
This is a multi-line comment,
consiting of more than one line.
---
```

### Imports
`import` statements are used to bring functions from [packages](/brane/packages/packages.html) and/or the [standard library](/brane/standard-library/standard-library.html) into scope:

```go
import "fs"
import "text"
```

### Variables
Variables are created with the assignment operator (`:=`). All variables are __immutable__, i.e. they can't be updated in-place. However, new variables can be declared using an existing name (shadowing).

```go
my_variable := "Hello, world!"
my_variable := 42
```

Global input parameters are specified by creating a new unkown (`??`) variable:

```go
my_input := ?? as Integer
```

Here we use the `as` keyword to indicate the variable's type, since it cannot be infered based on `??`.

### Arrays
Arrays, indicated using `[]`, can be created to contain multiple values of __the same type__, i.e. vectors:

```go
numbers := [1, 2, 3, 4, 5]
names := ["John", "Jane"]

my_a := "a"
my_b := "b"

my_values := [my_a, my_b]
```

### Objects
Objects in Bakery are __a collection of variables__ based on a custom type. These custom types can only be defined as part of [packages](/brane/packages/packages.html). To create an object, use the `new` keyword and specify the type:

```go
my_person := new Person { firstname: "John", lastname: "Smith" }
```

The individual variables of this object, i.e. properties, can be accessed using the `.` notation:

```go
import "text"

fullname := my_person.firstname + my_person.lastname
```

### Functions
Similar to custom types, functions can only be defined as part of [packages](/brane/packages/packages.html). When calling a function, the output can be assigned to a new variable. Consider the `split` function from the [standard library](/brane/standard-library/standard-library.html):

```go
import "text"

words := split "Have a good day!"
```

However, this is optional. Sometimes you just want to execute a function and ignore the output, i.e. fire-and-forget. Or perhaps the function doesn't return any value, in this case the return type is `Unit`.

### Conditionals
The flow of the program can be controlled using `if-else` conditionals:

```go
number := ?? as Integer

if number >= 10: 
    message := "Higher or equal"
else:
    message := "Lower"
```
The `else` clause is optional, and the clause body can also be written on the same line as the `if`:

```go
number := ?? as Integer
if number > 10: number := 10
```

The supported comparison operators are:

| Operator | Description                            | 
|:---------|:---------------------------------------|
| `>`      | Is the left-hand side greater than the right-hand side? |
| `<`      | Is the left-hand side less than the right-hand side? |
| `>=`     | Is the right-hand side greater or equal to the right-hand side? |
| `<=`     | Is the left-hand side less than or equal to the right-hand side?  |
| `!=`     | Are the left-hand side and right-hand side not equal? |
| `=`      | Are the left-hand side and right-hand side equal ? |

__Note__: equality is checked using the single `=` operator, not a double `==` operator.

### Loops
Bakery has support for two types of loops: `wait-until` and `while`.

The `while` loop will execute a chunk of code, as long as a certian condition is true:

```go
number := 1
while number < 10:
    number := number + 1

// Now the number is 10
```

The `wait-until` loop will halt, i.e. a __check-only loop__, the until a certain condition is true:

```go
transfer := transfer files to directory
wait until transfer status = "complete"

// The transfer is complete
```

<!-- 
## Notebook usage
Coming soon. -->