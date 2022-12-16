---
description: Programmable Orchestration of Applications and Networking
---

# The Brane Framework

Regardless of the context and rationale, running distributed applications on geographically dispersed IT resources often comes with various technical and organizational challenges. If not addressed appropriately, these challenges may impede development, and in turn, scientific and business innovation. We have developed Brane to support implementers in addressing these challenges. Brane utilizes containerization to encapsulate functionalities as portable building blocks. Through programmability, application orchestration can be expressed using an intuitive domain-specific language. As a result, end-users with limited programming experience are empowered to compose applications by themselves, without having to deal with the underlying technical details.

## Overview

A Brane instance starts out as a barebone infrastructure, with only a minimal set of functionalities. Using the tools from the Brane programming model, the instance can be **programmatically extended** to satisfy application-specific requirements. This is done by populating the instance's registry with custom functions, by means of creating **packages**. These functions can then be used, also in a programmatic manner, as building blocks for data processing pipelines.

{% content-ref url="broken-reference" %}
[Broken link](broken-reference)
{% endcontent-ref %}

The programming model has been designed with three roles in mind: domain scientists (end-users), software engineers, and system engineers. System engineers are concerned with the deployment and the operation of software, and thus are in charge of tuning and adding low-level functions, e.g. (optimized) data transfers. The software engineer add high-level functions, e.g. computational tasks, possibly reusing one ore more low-level functions. Finally, domain scientists use the available functions to compose data processing pipelines, without having to worry about the underlying details.

Brane ensures a **seamless integration** of custom functions through the programming model's tools.

{% content-ref url="broken-reference" %}
[Broken link](broken-reference)
{% endcontent-ref %}

Packages are used to **bundle newly developed functions**, and make them available to a Brane instance. Docker images are used to make packages **self-contained**, i.e. they contain all the required dependencies, files, and metadata. Brane offers two distinct package builders. Each builder targets a different way of implementing functions. In the end, all created packages share a uniform interface.

{% content-ref url="broken-reference" %}
[Broken link](broken-reference)
{% endcontent-ref %}
