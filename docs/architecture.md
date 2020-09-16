---
layout: default
title: Architecture
nav_order: 2
description: "Architecture"
permalink: /architecture
---

# Architecture
Conceptually, Brane consists of two parts: a programming model and a runtime system.

## Programming Model
Initially, a deployment of the runtime system starts out with only a minimal set of functionalities. 
With the tools provided by the programming model, the runtime system can, programmatically, be molded based on use-case specific requirements. Once the system is extended with the desired functionalities, the programming model can be used to (interactivly) run workflows and services on the runtime system.

For the above, Brane assumes a separation of concerns between users with different roles. Typically we distinguish: domain experts, domain scientists, research engineers, and system engineers. Domain experts and system engineers will contribute lower-level functionalities, e.g. algorithms and (optimized) data transfers. The research engineers will focus on higher-level functionalities, e.g. multi-step routines, possibly integrating one or more lower-level functionalities. Domain scientists will get involved after the system has been fully developed, in order to create and run workflows and/or services. Of course, this exemplary division is not cut into stone nor in any way technically enforced, any variation is possible.

By using the tooling of the programming model, the interoperability between contributed functionalities is ensured automatically. This is not only beneficial technically, i.e. it relieves developers of a tedious task. But also organizationally, as functionality can be developed independently in an isolated fashion. This mechanism, thus, also facilitates collaboration between organizations with divided responsibilities.

<p style="text-align: center">
    <img src="/assets/img/programming-model.png" alt="architecture">
</p>

In the next sections, four elements of the programming model will be described in more detail.

### Packages
The way 

- different types
- packaged as Docker images: allows signing and deployment + ease of use for dev side.
- registry
- executing packages

### Bakery
... 

- prefix, infix, postfix notation
- output is instructions

### Instructions
...

- i.e. bytecode
- allows the runtime system to be used by other languages too, e.g.

### Jupyter Notebooks
...

## Runtime System
...

<p style="text-align: center">
    <img src="/assets/img/runtime-system.png" alt="architecture">
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

