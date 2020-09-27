---
layout: default
title: Overview
nav_order: 2
description: "Overview"
permalink: /overview
---

# Overview
On this page, we briefly describe Brane's __programming model__ and __runtime system__.

## Programming model
By default, a new instantiation of the Brane runtime system starts out as a barebone infrastructure, with only a minimal set of functionalities. 
Using the tools from the Brane programming model (Fig. 1), the runtime system can be __programmatically extended__ to satisfy application-specific requirements. This is done by populating the runtime system's [registry](#registry) with custom functions. These functions can then be used, also in a programmatic manner, as building blocks for data processing pipelines.

The programming model has been designed with three roles in mind: domain scientists (end-users), application developers, and system engineers. System engineers are concerned with the deployment and the operation of software, and thus are in charge of tuning and adding low-level functions, e.g. (optimized) data transfers. The application developers add high-level functions, e.g. computational tasks, possibly reusing one ore more low-level functions. Finally, domain scientists use the available functions to compose data processing pipelines, without having to worry about the underlying details.

Brane ensures a __seamless integration__ of custom functions through the programming model's tools. 

<p style="text-align: center">
    <img src="/brane/assets/img/programming-model.svg" width="500px" alt="The components of the Brane programming model.">
    <br/>
    <sup>Figure 1: components of the Brane programming model</sup>
</p>

In the next sections, the four main components of the programming model are highlighted, namely: packages, Bakery, bytecode, and Jupyter notebooks. The <abbr title="Command-line interface">CLI</abbr> and <abbr title="Read-eval-print loop">REPL</abbr> are discussed as part of the [quickstart](/brane/quickstart/quickstart.html). Docker images are used as provided by Docker, described [here](https://docs.docker.com/get-started/overview/#docker-objects). The interoperability layer is a purely conceptual distinction: above is for users, below is what the runtime system operates on.

### Packages
Packages are used to __bundle newly developed functions__, and make them available to the runtime system. Docker images are used to make packages __self-contained__, i.e. they contain all the required dependencies, files, and metadata. Brane offers four distinct package builders. Each builder targets a different way of implementing functions. In the end, all created packages share a uniform interface.

The package builders are:

| Builder  | Functions                                     | 
|:---------|:------------------------------------------------|
| ECU      | functions created from (existing) arbitrary code. |
| CWL      | functions created from workflows described in <abbr title="Common Workflow Language">CWL</abbr> format. |
| OAS      | functions created from Web APIs described in OpenAPI specification. |
| DSL      | functions created from Bakery DSL scripts. |

In order to execute functions from packages, Brane embeds a proxy in the package at build time. This proxy acts as a bridge between the package its functions and the runtime system. 
It is responsible for executing the function's underlying implementation with the right arguments, and sending back the function's output to the runtime system.
The exception being DSL packages, these packages do not require a self-contained Docker image: their functions can be run directly by the runtime system.

See the [Packages](/brane/packages/packages.html) page for more information.

### Bakery
Bakery is the <abbr title="Domain-specific language">DSL</abbr> of the Brane framework. It's purpose, inspired by [Cookery](https://github.com/mikolajb/cookery), is to express a complete data processing pipeline __as simple as a cooking recipe__. This makes Bakery programs easy to reason about and well-suited for domain scientists with no or limited programming experience. The Bakery language supports __basic programming constructs__ like variables, conditionals, loops and functions. Statements follow a English sentence-like structure, and typically can be written using only one line.

See the [Bakery](/brane/bakery) page for more information.

### Bytecode
Bakery programs are compiled into bytecode, i.e. an __intermediate representation__, interpretable by the runtime system. In the case of Brane, bytecode takes the form of a graph of instructions (Fig. 2). 


<p style="text-align: center">
    <img src="/brane/assets/img/instructions.svg" width="400px" alt="Exemplary bytecode.">
    <br/>
    <sup>Figure 2: exemplary bytecode</sup>
</p>

The use of a bytecode __decouples__ the programming model from the runtime system, paving the way for alternative and/or third-party programming models.
Four different instructions are available:

| Kind  | Description                                     | 
|:------|:------------------------------------------------|
| Act   | Performs a function, optionally assigning the output to a variable. |
| Mov   | Performs, conditionally, a move other than the default forward move. |
| Sub   | Steps into a substructure, i.e. a nested graph of instructions. |
| Var   | Performs basic variable manipulation, i.e. assignment and retreival. |

### Jupyter notebooks
Jupyter notebooks are widely used among domain scientists. These notebooks are the recommended way of composing data processing pipelines with Bakery. It provides an additional layer of interaction through notebook widgets, wich also can be utilized for __real-time, visual, monitoring of progress__. 

Brane offers a custom version of [JupyterLab](https://jupyterlab.readthedocs.io/en/stable)  (Fig. 3), featuring a Bakery [kernel](https://jupyter.readthedocs.io/en/latest/projects/kernels.html) and registry browser.

<p style="text-align: center">
    <img src="/brane/assets/img/notebook.png" style="margin-bottom: -25px" width="600px" alt="Exemplary notebook.">
    <br/>
    <sup>Figure 3: exemplary notebook</sup>
</p>

## Runtime system
The runtime system is the engine that __performs the orchestration__ of the data processing pipelines created by domain scientists. Individual (sub)steps, i.e. custom functions, can be executed on the local infrastructure or on remote infrastructures, e.g. the Cloud and/or <abbr title="High-performance computing">HPC</abbr> clusters. __Heterogeneity__ of the underlying infrastructure(s) is handled by the runtime system. For this, the Brane framework relies heavily on __virtualization and container runtimes__. For instance, Brane is capable of converting Docker images to other image formats to support execution of packaged functions on infrastructures with container runtimes other than Docker, e.g. [Singularity](https://sylabs.io/guides/3.6/user-guide) and [Charliecloud](https://hpc.github.io/charliecloud), such as <abbr title="High-performance computing">HPC</abbr> clusters.

<p style="text-align: center">
    <img src="/brane/assets/img/runtime-system.svg" width="500px" alt="The Brane runtime system.">
    <br/>
    <sup>Figure 4: the components of the Brane runtime system.</sup>
</p>

In the next sections, the components of the runtime system will be discussed in more detail: registry, API, relay, vault, virtual machine, and the event loop. [Kubernetes](https://kubernetes.io) and [Xenon](https://xenon-middleware.github.io/xenon/) are described elsewhere.

### Registry
The registry stores the packages that have been added to the runtime system. In addition, it provides __indexed access to the metadata__ of these packages. This includes information such as: functions contained in a package; the input and output parameters of each function; and package descriptions and documentation, if available. The __Bakery compiler__ relies on the metadata provided by the registry to determine which functions can be used within Bakery programs and data processing pipelines. 


### API
The API is the main entrypoint to the runtime system. It provides endpoints to manage the execution of data processing pipelines. To start an execution, first a __session__ has to be created through the API. Then, chunks of bytecode, i.e. compiled Bakery programs, can be executed by creating __invocations__. __Variables__ created during an invocation are also available to subsequent invocations, they scoped to the parent session. The API also has endpoints to pause, resume and stop invocations.

### Relay
__Callbacks__, e.g. outputs from functions executed on a remote infrastructure, to the runtime system go through the relay service. __Telemetry data__ also enters the runtime system through the relay service.

### Vault
The vault, implemented using [HashiCorp Vault](https://www.vaultproject.io), provides __secure store for secrets__, e.g. credentials, certificates, and/or tokens. When a custom function requires one ore more secrets. They should be specified as a special secret (input) parameters of the function. During runtime, these secrets will be added to the function's arguments just before execution. This __implicit variable__ mechanism prevents embedding hard-coded secrets directly within data processing pipeline applications.

### Virtual machine
...


- executes the instruction graph
- uses Redis for working copy variables, temps
- diagram with libraries: brane-sys for diff. environments, swaps out for others.

### Event loop
... 

- is at the center, makes everything tick
- diagram ?