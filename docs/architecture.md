---
layout: default
title: Architecture
nav_order: 2
description: "Architecture"
permalink: /architecture
---

# Architecture
In this section, we briefly describe Brane's __programming model__ and __runtime system__.

## Programming model
A new Brane runtime system starts out as a barebone, with only a minimal set of functionalities. 
Using the tools from the Brane programming model (Fig. 1), the functionality of the runtime system can be extended to satisfy application-specific requirements. This is done by populating the Brane [registry](#registry) with custom functions. Which then can be used as building blocks for data processing pipelines.

The programming model has been designed with three roles in mind: domain scientists (end-users), application developers, and system engineers. System engineers are concerned with the deployment and the operation of software, and thus are in charge of tuning and adding low-level functions of/to the runtime system, e.g. (optimized) data transfers. The application developers can add high-level functions to the runtime system, possibly reusing one ore more low-level functions. Finally, domain scientists use the available functions to create data processing pipelines, i.e. workflows and services, preferably without having to worry about the low-level details.

Brane ensures a seamless integration of the functions developed across the three roles. The low-level functionalities developed by system engineers are the foundation for the application developers. And, the functionalities developed by the application developers are starting point for domain scientists.

<p style="text-align: center">
    <img src="/brane/assets/img/programming-model.svg" width="500px" alt="The Brane programming model.">
    <br/>
    <sup>Figure 1: the components of the Brane programming model.</sup>
</p>

In the next sections, the four main components of the programming model will be discussed in more detail: packages, Bakery, instructions, and Jupyter notebooks. The <abbr title="Command-line interface">CLI</abbr> and <abbr title="Read-eval-print loop">REPL</abbr> are described in the [quickstart](/brane/quickstart/quickstart.html). Docker images are used as provided by Docker, described [here](https://docs.docker.com/get-started/overview/#docker-objects). The interoperability layer is a conceptual distinction: above is for users, below is what the runtime system operates on.

### Packages
Packages are used to bundle newly developed functions and make them compatible with the runtime system. Docker images are used to make packages self-contained, i.e. they contain all the required system dependencies, files and metadata. Four distinct package builders are provided by the Brane framework, each to support a typical function that can be added to the runtime system. These are:

- functions created to call Web APIs, based on an OpenAPI specification;
- functions created from existing (arbitrary) source code;
- functions created as the result of combining other functions (using [Bakery](#bakery));
- functions described as a <abbr title="Common Workflow Language">CWL</abbr> workflow or command-line tool.

To execute a function from a package, Brane adds a proxy to the package, i.e. Docker image, when it is built. This proxy acts as a bridge between the package's functions and the runtime system. 
It is responsible for invoking the function's code, depending on the package, with the right arguments and returning the output to the runtime system.
The exception being [Bakery](#bakery) packages, these packages do not require a self-contained Docker image: their functions are run within the runtime system.

See the [Packages](/brane/packages/packages.html) page for more information.

### Bakery
Bakery is the <abbr title="Domain-specific language">DSL</abbr> of the Brane framework, is inspired by [Cookery](https://github.com/mikolajb/cookery). It's purpose is to express a data processing as simple as a cooking recipe. This makes Bakery programs easy to reason about and well-suited for domain scientists with no or limited programming experience. Bakery supports basic programming constructs like variables (typed), conditionals, loops and function calls. Statements follow a English sentence-like structure, and typically can be written using only one line.

See the [Bakery](/brane/bakery) page for more information.

### Instructions
Bakery programs are compiled into a graph of instructions (Fig. 2). These instructions are an intermediate representation that the runtime system understands. It decouples the programming model from the runtime system, paving the way for alternative, third-party, programming models.

<p style="text-align: center">
    <img src="/brane/assets/img/instructions.svg" width="400px" alt="Exemplary graph of instructions.">
    <br/>
    <sup>Figure 2: exemplary graph of instructions.</sup>
</p>

Different kind of instructions exist:

| Kind  | Description                                     | 
|:------|:------------------------------------------------|
| Act   | Performs a function, optionally assigning the output to a variable. |
| Mov   | Performs, conditionally, a move other than the default forward. |
| Sub   | Steps into a substructure, i.e. a nested graph of instructions. |
| Var   | Performs basic variable manipulation, i.e. assignment and retreival. |

### Jupyter notebooks
JupyterLab is a widely used <abbr title="Integrated development environment">IDE</abbr> among domain scientists. Its notebooks are the recommended way of writing Bakery programs. Users can also interact with the runtime system through the widgets within notebook. For example, visually monitor execution progress, and display monitoring statistics. Brane includes a custom version of [JupyterLab](https://jupyterlab.readthedocs.io/en/stable), with a Bakery [kernel](https://jupyter.readthedocs.io/en/latest/projects/kernels.html) and registry browser installed (Fig. 3).

<p style="text-align: center">
    <img src="/brane/assets/img/notebook.png" style="margin-bottom: -25px" width="600px" alt="Exemplary Jupyter notebook.">
    <br/>
    <sup>Figure 3: exemplary Jupyter notebook.</sup>
</p>

## Runtime system
The runtime system (Fig. 4) is the engine that runs applications. Applications can be executed as part of the runtime system on the local resources, or executed on remote infrastructures, e.g. the Cloud or <abbr title="High-performance computing">HPC</abbr> clusters. Brane can work on heterogenous computing infrastructures: if necessary, it will convert the packages, which are based on Docker images, to image format of the target container runtime.

<p style="text-align: center">
    <img src="/brane/assets/img/runtime-system.svg" width="500px" alt="The Brane runtime system.">
    <br/>
    <sup>Figure 4: the components of the Brane runtime system.</sup>
</p>

In the next sections, the five main components of the runtime system will be described in more detail: API, relay, registry, vault and the VM. The other components are documented elsewhere. [Singularity](https://sylabs.io/guides/3.6/user-guide) and [Charliecloud](https://hpc.github.io/charliecloud) are container runtimes, supported as provided by their respective vendors. [Xenon](https://xenon-middleware.github.io/xenon/) is a middleware. [Redis](https://redis.io) and [PostgreSQL](https://www.postgresql.org) are the used data stores. [Kubernetes](https://kubernetes.io) is a container platform.

### API
...

- entrypoint for runtime system
- session with one ore more invocations, instructions (diagram)
- persistence is stored in PostgreSQL

### Relay
...

- used for callbacks to the runtime system, e.g. outputs from the proxy, i.e. functions.

### Registry
...

### Vault
...

- stores secrets etc, side-loading / implicit variables.

### VM
...

- executes the instruction graph
- uses Redis for working copy variables, temps
- diagram with libraries: brane-sys for diff. environments, swaps out for others.

