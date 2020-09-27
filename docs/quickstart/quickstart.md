---
layout: default
title: Quickstart
nav_order: 4
has_children: true
---

# Quickstart
_This quickstart assumes you have [installed](/brane/installation.html) both the <abbr title="Command-line interface">CLI</abbr> and the backend._

In this quickstart you will learn how to add functionalities to the Brane runtime system, and how to use them as building blocks for data processing pipelines. You'll learn this while implementing a word count application (Fig. 1). This basic example application retreives the README.md file from a GitHub repository, determines if it meets a certain word count threshold, and then outputs a text accordingly.

<p style="text-align: center">
    <img src="/brane/assets/img/word-count.svg" width="400px" alt="The flow of the word counta application.">
    <br/>
    <sup>Figure 1: illustration of the word count application.</sup>
</p>

We have designed this quickstart to explain the development flow in Brane: first add the required functions to Brane by means of creating packages. Then write the final application in Bakery <abbr title="Domain-specific language">DSL</abbr>.

The steps of this quickstart are categorized based on the performed tasks, and associated to the typical roles within Brane's [programming model](/brane/architecture#programming-model). There are three categories:

- <span class="label label-green">Users</span>: write an application (domain scientists).
- <span class="label label-blue">Application</span>: contribute use-case specific functions (application developers).
- <span class="label label-red">System</span>: contribute generic or system related functions (system engineers).

The code and files used can be found in the [examples](https://github.com/onnovalkering/brane/tree/master/examples/wordcount) directory.

[Next](/brane/quickstart/1-retreive-readme.html){: .btn .btn-outline .flex-justify-end }
