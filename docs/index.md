---
layout: default
title: Home
nav_order: 1
description: "Index"
permalink: /
---

# The Brane Framework

Programmable Orchestration of Applications and Networking
{: .fs-6 .fw-300 }

[Get started now](/brane/quickstart/quickstart.html){: .btn .btn-primary .fs-5 .mb-4 .mb-md-0 .mr-2 } 
[See it in action](/brane/demonstrations/demonstrations.html){: .btn .fs-5 .mb-4 .mb-md-0 }

---

## Introduction
Scientific endeavors are demanding, with acceleration, ever more storage and computing capabilities. Projects exist that even desire the next frontier of computing: exascale (10<sup>18</sup> FLOPS). Still, at present, the first supercomputer capable of exascale computing has yet to become operational. It will certainly take years before exascale becomes the widespread norm for high-performance computing (HPC).

Since we, currently and in the short-term, cannot make use of centralized exascale capabilities, we need to resort to a hybrid of HPC clusters, the Cloud and distributed data stores to meet the demand for exascale as much as possible. This brings myriad challenges, not only at all levels of the technical stack but also organizationally, e.g. due to distributed collaboration with divided responsibilities. The existing solutions, i.e. the typical web portal backed by a workflow management system (WfMS), do not provide sufficient control to address all of the aforementioned challenges. For the most part, this is because these WfMSs do not let us model and/or optimize the underlying physical infrastructure(s) and network(s) directly, which is crucial in establishing extreme-scale deployments. Moreover, with these semi-static web portals, domain scientists are often dependent on others for maintaining their applications. This often results in long development cycles, which in turn hampers scientific progress.

With the Brane framework we address these shortcomings. It features a programmatic approach to constructing research infrastructures that is intuitive and easy to use, yet is expressive enough to capture and control the entire, distributed, technical stack. The programming model is based on the separation of concerns principle. For each level of the technical stack, and associated roles, different tooling and abstractions are provided. As a result, top-level applications can be written in a <abbr title="Domain-specific language">DSL</abbr> by domain scientists, while underlying (optimized) routines are implemented by the relevant experts.

Learn more about how Brane works on the [Overview](/brane/overview.html) page.

## Features

Brane features...

- a simple <abbr title="Domain-specific language">DSL</abbr> that can be used with limited to none programming experience;
- a programmatic approach to constructing research infrastructures;
- interactive computing, with a detach/attach mechanisms and visual monitoring.
- a performant event-driven runtime, based on microservices;

---

## About the project
[![Release](https://img.shields.io/github/release/onnovalkering/brane.svg)](https://github.com/onnovalkering/brane/releases/latest)
[![DOI](https://zenodo.org/badge/258514017.svg)](https://zenodo.org/badge/latestdoi/258514017)
{: .no-lnk }

Brane &copy; 2020 University of Amsterdam

<a class="no" href="https://mns-research.nl" target="_blank">
    <img src="/brane/assets/img/mns.png" width="150px" alt="MNS">
</a>

_The Brane framework has been developed as part of [PROCESS](https://www.process-project.eu), which has received funding from the European Union's Horizon 2020 research and innovation programme under grant agreement 777533._

### License

Brane is licensed under the [Apache-2.0](https://github.com/onnovalkering/brane/blob/master/LICENSE) license.