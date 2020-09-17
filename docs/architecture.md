---
layout: default
title: Architecture
nav_order: 2
description: "Architecture"
permalink: /architecture
---

# Architecture
Brane consists of two parts: a programming model (Fig. 1) and a runtime system (Fig. 2).

## Programming Model
Initially, the runtime system, i.e. a [Brane instance](/brane/installation#instance), starts out with only a minimal set of functionalities. 
With the tools provided by the programming model, the runtime system can, programmatically, be molded based on use-case specific requirements. This is done by populating the runtime system's registry with custom functions. And, after that, (interactively) developing workflows and/or services.

During the above, the programming model assumes a separation of concerns between users based on their role. Typically we distinguish the following roles: domain experts, domain scientists, research engineers, and system engineers. Domain experts and system engineers will contribute lower-level functions, e.g. algorithms and (optimized) data transfers. The research engineers are responsible for the higher-level functions, possibly reusing one ore more lower-level functions. Once a sufficient set of functions is available, the domain scientists will use these as building blocks for workflows and/or services. This seperation is not cut into stone nor in any way enforced, any variation is possible.

Through usage of Brane's tooling, the interoperability between contributed functions, which may be heterogenous in implementation, is guaranteed automatically. To ensure this, the programming model imposes a set of constrains. For instance, the input and output parameters of functions must conform to Brane's (extendable) type system. Also, how to execute a particular function must be made explicit for Brane. More constrains apply, these will be mentioned in the relevant sections. This approach to interoperability is not only beneficial technically, i.e. it relieves developers of a tedious task. But, since functions can be developed independently, also organizationally. When organizations collaboratively build infrastructures based on the Brane framework, each can contribute functions based on their expertise, in an isolated manner if desired, using the technologies that they find most appropriate.

<p style="text-align: center">
    <img src="/brane/assets/img/programming-model.svg" width="500px" alt="The Brane programming model.">
    <br/>
    <sup>Figure 1: the elements of the Brane programming model.</sup>
</p>

In the next sections, four elements of the programming model will be described in more detail, namely packages, Bakery, instructions, and Jupyter notebooks. The CLI and REPL are discussed as part of the [quickstart](/brane/quickstart). Docker images are used as provided by Docker, described [here](https://docs.docker.com/get-started/overview/#docker-objects). The interoperability layer is a conceptual distinction: above is for users, below is what the runtime system operates on.

### Packages



The runtime system is extended with functionality through packages. Each package adds one or more functions to the system, functions to the system

- different types
- packaged as Docker images: allows signing and deployment + ease of use for dev side.
- registry
- executing packages 

See the [Packages](/brane/packages/packages.html) page for in-depth guides for each kind of package.

### Bakery
... 

- prefix, infix, postfix notation
- output is instructions
- compile time checks of the workflow and services (prevents expensive runs)

See the [Bakery](/brane/bakery) page for 

Bakery is described 

### Instructions
...

- i.e. bytecode
- allows the runtime system to be used by other languages too, e.g.

### Jupyter Notebooks
...

## Runtime System
...

- run time interoperability

<p style="text-align: center">
    <img src="/brane/assets/img/runtime-system.svg" width="500px" alt="The Brane runtime system.">
    <br/>
    <sup>Figure 2: the elements of the Brane runtime system.</sup>
</p>

In the next sections, four elements of the runtime system will be described in more detail.

### API
...

### Relay
...

### Vault
...

### VM
...

