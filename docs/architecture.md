---
layout: default
title: Architecture
nav_order: 2
description: "Architecture"
permalink: /architecture
---

# Architecture
Conceptually, Brane consists of two parts: a __programming model__ and a __runtime system__.

## Programming model
Initially, the runtime system starts out as a barebone with only a minimal set of functionalities. 
With the tools provided by the programming model, the runtime system can, programmatically, be molded based on use-case specific requirements. This is done by populating the runtime system's registry with custom functions. And, after that, (interactively) developing workflows and/or services.

During the above, the programming model assumes a separation of concerns between users based on their role. Typically we distinguish the following roles: domain experts, domain scientists, research engineers, and system engineers. Domain experts and system engineers will contribute lower-level functions, e.g. algorithms and (optimized) data transfers. The research engineers are responsible for the higher-level functions, possibly reusing one ore more lower-level functions. Once a sufficient set of functions is available, the domain scientists will use these as building blocks for workflows and/or services. This seperation is not cut into stone nor in any way enforced, any variation is possible.

Through usage of Brane's tooling, the interoperability between contributed functions, which may be heterogenous in implementation, is guaranteed automatically. To ensure this, the programming model imposes a set of constrains. For instance, the input and output parameters of functions must conform to Brane's (extendable) type system. Also, how to execute a particular function must be made explicit for Brane. More constrains apply, these will be mentioned in the relevant sections. This approach to interoperability is not only beneficial technically, i.e. it relieves developers of a tedious task. But, since functions can be developed independently, also organizationally. When organizations collaboratively build infrastructures based on the Brane framework, each can contribute functions based on their expertise, in an isolated manner if desired, with the technology stack that they find most appropriate.

<p style="text-align: center">
    <img src="/brane/assets/img/programming-model.svg" width="500px" alt="The Brane programming model.">
    <br/>
    <sup>Figure 1: the elements of the Brane programming model.</sup>
</p>

In the next sections, four elements of the programming model will be discussed in more detail: packages, Bakery, instructions, and Jupyter notebooks. The <abbr title="Command-line interface">CLI</abbr> and <abbr title="Read-eval-print loop">REPL</abbr> are described as part of the [quickstart](/brane/quickstart/quickstart.html). Docker images are used as provided by Docker, described [here](https://docs.docker.com/get-started/overview/#docker-objects). The interoperability layer is a conceptual distinction: above is for users, below is what the runtime system operates on.

### Packages
Packages are used to bundle functions and as a carrier towards the runtime system. Docker images are used to make packages self-contained, i.e. they contain the required system dependencies, files and metadata. Several distinct builders are available to create packages, resulting in packages of a different kind. However, when it comes to functions, all packages share the same uniform interface.

The package builders are:

| Kind  | Description                                     | 
|:------|:------------------------------------------------|
| [CWL](/brane/packages/cwl.html)    | Builds packages for workflows described with the [CWL](https://www.commonwl.org/v1.1/) specification. |
| [DSL](/brane/packages/dsl.html)    | Builds packages based on [Bakery](/brane/bakery) scripts. |
| [ECU](/brane/packages/ecu.html)    | Builds packages based on arbitrary code. |
| [OAS](/brane/packages/oas.html)    | Builds packages for Web APIs described with the [OpenAPI](http://spec.openapis.org/oas/v3.0.3) specification. |
 
To be able to execute a function from a package, Brane adds a proxy (`brane-init`) to the package, i.e. Docker image. This proxy acts as a bridge between the package's functions and the runtime system. 
It is responsible for invoking the function's code, depending on the package, with the right arguments and returning the output to the runtime system.
The exception being <abbr title="Domain-specific Language">DSL</abbr> packages, these packages do not require a self-containted Docker image: they can be run directly within the runtime system.

See the [Packages](/brane/packages) page for more information.

### Bakery
Bakery is Brane's <abbr title="Domain-specific Language">DSL</abbr>. It has been designed, influenced by [Cookery](https://github.com/mikolajb/cookery), to have a low learning curve and to read as easy to follow recipes. This makes Bakery programs easy to reason about and accessible for users with no or limited programming experience. Bakery supports variables (typed), conditionals, loops and function calls. Statements follow a sentence-like structure, and typically take only one line.

The syntax of a function call is specified by the author of the target function. The author specifies a pattern using a pre-/in-/postfix notation. The same pattern can be used by different functions as long as the arguments have different types, i.e. overloading. This syntax mechanism substantiates the sentence-like statements, and allows Bakery to be customized towards domain-specific jargon.

See the [Bakery](/brane/bakery) page for more information about syntax and semantics.

### Instructions
The compilation output of a Bakery program is a graph of instructions (Fig. 2). These instructions are an intermediate representation that the runtime system understands. It decouples the programming model from the runtime system, paving the way for alternative, third-party, programming models.

<p style="text-align: center">
    <img src="/brane/assets/img/instructions.svg" width="400px" alt="Exemplary graph of instructions.">
    <br/>
    <sup>Figure 2: exemplary graph of instructions.</sup>
</p>

Four kinds of instructions exists:

| Kind  | Description                                     | 
|:------|:------------------------------------------------|
| Act   | Performs a function, optionally assigning the output to a variable. |
| Mov   | Performs, conditionally, a move other than the default forward. |
| Sub   | Steps into a substructure, i.e. a nested graph of instructions. |
| Var   | Performs basic variable manipulation, i.e. assignment and retreival. |

### Jupyter notebooks
Notebooks are the recommended way of writing Bakery programs. It is likely to be a familiar <abbr title="Integrated development environment">IDE</abbr> for domain scientists. Moreover, it allows the utilization of rich widgets for interaction with the runtime system, visualization of progress, and the display of monitoring statistics. The programming model includes a custom version of [JupyterLab](https://jupyterlab.readthedocs.io/en/stable), with a Bakery [kernel](https://jupyter.readthedocs.io/en/latest/projects/kernels.html) and registry browser installed (Fig. 3).

<p style="text-align: center">
    <img src="/brane/assets/img/notebook.png" style="margin-bottom: -25px" width="600px" alt="Exemplary Jupyter notebook.">
    <br/>
    <sup>Figure 3: exemplary Jupyter notebook.</sup>
</p>

## Runtime system


- as mentioned before ... this is what "runs" workflows and services
- it aims to: when it runs locally, it runs elsewhere too: constrains / interop. (one port)
- run Docker images -> conver to container runtime in place.
- event-driven architecture based on Kafka.
- package of different type: remote, compute, determines where to execute

<p style="text-align: center">
    <img src="/brane/assets/img/runtime-system.svg" width="500px" alt="The Brane runtime system.">
    <br/>
    <sup>Figure 4: the elements of the Brane runtime system.</sup>
</p>

- text
- text 
- text

- sequence diagram

In the next sections, four elements of the runtime system will be described in more detail: API, relay, vault and the VM. The other elements are documented elsewhere. The registry, used to store Docker images, is provided by Docker and documented [here](https://docs.docker.com/registry). The [Singularity](https://sylabs.io/guides/3.6/user-guide) and [Charliecloud](https://hpc.github.io/charliecloud) runtimes are supported as provided by their respective vendors. The [Xenon](https://xenon-middleware.github.io/xenon/) middleware is developed by <abbr title="Netherlands eScience Center">NLeSC</abbr>.

### API
...

- entrypoint for runtime system
- session with one ore more invocations, instructions (diagram)
- persistence is stored in PostgreSQL

### Relay
...

- used for callbacks to the runtime system, e.g. outputs from the proxy, i.e. functions.

### Vault
...

- stores secrets etc, side-loading / implicit variables.

### VM
...

- executes the instruction graph
- uses Redis for working copy variables, temps
- diagram with libraries: brane-sys for diff. environments, swaps out for others.
