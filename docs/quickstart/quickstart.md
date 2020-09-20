---
layout: default
title: Quickstart
nav_order: 4
has_children: true
---

# Quickstart
_This quickstart assumes you have [installed](/brane/installation.html) both the <abbr title="Command-line interface">CLI</abbr> and the backend._

In this quickstart you'll become aquinted with the basics of the Brane framework. We'll implement a basic word count application (Fig. 1). This application retreives the README.md file from a GitHub repository, determines if it meets a certian word count threshold, and then outputs a text accordingly.

We follow Brane's advocated development flow. First, we'll add the required functions to Brane by means of creating packages. Then, we program the final application, using the functions and a DSL.

<p style="text-align: center">
    <img src="/brane/assets/img/word-count.svg" width="400px" alt="The flow of the word counta application.">
    <br/>
    <sup>Figure 1: illustration of the word count application.</sup>
</p>

The steps of this quickstart are categorized based on the performed tasks, and associated to the typical roles within Brane's [programming model](/brane/architecture#programming-model). There are three categories:

- <span class="label label-green">Users</span>: write an application (domain scientists).
- <span class="label label-blue">Application</span>: contribute use-case specific functions (domain experts or research engineers).
- <span class="label label-red">System</span>: contribute generic or system related functions (system engineers).

The code and files used can be found in the [examples](https://github.com/onnovalkering/brane/tree/master/examples/wordcount) directory.

[Next](/brane/quickstart/1-retreive-readme.html){: .btn .btn-outline .flex-justify-end }
