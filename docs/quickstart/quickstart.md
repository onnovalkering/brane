---
layout: default
title: Quickstart
nav_order: 4
has_children: true
---

# Quickstart
_This quickstart assumes you have [installed](/brane/installation.html) both the <abbr title="Command-Line Interface">CLI</abbr> and the backend._

During this quickstart you'll become aquinted with the basics of the Brane framework. We'll implement a basic word count application (Fig. 1). This application retreives a README.md from GitHub, then determines if it meets a certian word count threshold, and outputs a text accordingly.

- * we first create the functions (packages), then add these functions to the system (i.e. backend) and then write our final application. This is the advocated flow for using Brane.

<p style="text-align: center">
    <img src="/brane/assets/img/word-count.svg" width="400px" alt="The flow of the word counta application.">
    <br/>
    <sup>Figure 1: illustration of the word count application.</sup>
</p>

The steps of this quickstart are categorized based on the tasks performed, and (loosely) associated to the typical roles within Brane's [programming model](/brane/architecture#programming-model). The categories are as follows:

- <span class="label label-green">Users</span>: write an application (domain scientists).
- <span class="label label-blue">Application</span>: contribute use-case specific functions (domain experts or research engineers).
- <span class="label label-red">System</span>: contribute generic functions (system engineers).

The code and files used can be found in the [examples](https://github.com/onnovalkering/brane/tree/master/examples/wordcount) directory of Brane's GitHub repository.

[Next](/brane/quickstart/1-retreive-readme.html){: .btn .btn-outline .flex-justify-end }
