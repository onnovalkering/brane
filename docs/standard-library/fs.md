---
layout: default
title: FS
parent: Standard library
nav_order: 1
---

# FS
This package contains functions to interact with the file system.

```go
import "fs"
```

### new_directory
Creates a new directory, with a random name.
```go
mydir := new_directory
```

### new_directory_name
Creates a new directory, with the specified name.
```go
mydir := new_directory "mydir"
```

### new_directory_in
Creates a new directory, with the specified name, and inside the specified directory.

```go
mydir := new_directory
mysub := new_directory "mysub" in mydir
```

### new_file
Creates a new file, with a random name.
```go
myfile := new_file
```

### new_file_name
Creates a new file, with the specified name.
```go
myfile := new_file "myfile"
```

### new_file_in
Creates a new file, with the specified name, and inside the specified directory.
```go
mydir := new_directory "mydir"
myfile := new_file "myfile" in mydir
```

### new_temp_directory
Creates a new directory, with a random name, and marked as temporary.
```go
my_dir := new_temp_directory
```

### new_temp_file
Creates a new file, with a random name, and marked as temporary.
```go
my_file := new_temp_file
```