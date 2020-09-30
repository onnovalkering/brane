---
layout: default
title: Demonstration
nav_order: 5
description: "demonstration"
permalink: /demonstration
---

<link href="https://vjs.zencdn.net/7.8.4/video-js.css" rel="stylesheet" />

# Demonstration
This demonstration is based on a real-world data pipeline (Fig. 1) from the [LOFAR](http://lofar.org/about-lofar/general-information/introduction.html). This data pipeline generates, or rather calibrates, sky maps based on astronomical observations stored in LOFAR's <abbr title="Long-term archive">LTA</abbr>. Sky maps are images of the sky that aren't focused on a specific target. Astronomers use these sky maps for exploratory research and serendipitous discoveries. However, the pipeline is rather complex [[spreeuw2019lta](#)]. Partly because the typical sizes of astronomical observations, up to TBs, are non-trivial to handle. But also because running the necessary compute routines require domain knowledge and experience with <abbr title="High-performance computing">HPC</abbr> clusters. Due to this complexity, the utilization of the pipeline is minimal. 

The steps of this pipeline are as follows:

1. The input files must be staged. This is required since <abbr title="Long-term archive">LTA</abbr> files are stored on tape drives that\
are not directly accessible. During staging, the files are copied to a directly accessible cache.
2. Once the files are staged, they will be downloaded to a compute site, e.g. an <abbr title="High-performance computing">HPC</abbr> cluster.
3. Next, a number of [calibration tasks](https://support.astron.nl/LOFARImagingCookbook/factor.html) will be run, direction-indepenent and direction-dependent.
4. Finally, the output of these calibration routines is display to the astronomers.

In this demonstration we limit the number of calibration tasks (only [prefactor](https://github.com/lofar-astron/prefactor)).

<p style="text-align: center">
    <img src="/brane/assets/img/lofar-pipeline.png" width="675px" alt="The LOFAR calibration pipeline.">
    <br/>
    <sup>Figure 1: the LOFAR calibration pipeline.</sup>
</p>


With Brane, astronomers, i.e. the domain scientists, are relieved of the technical complexities. Based on the Brane's [programming model](#), domain experts and research engineers will take on the task of capturing the required domain knowledge and application specific functions. The technicalities of using <abbr title="High-performance computing">HPC</abbr> clusters and optimizing data transfers for TBs will be handled by the system engineers.

## Packages
We've created two packages for this demonstration: `lta` for staging and downloading; and `prefactor` to perform calibration tasks. The `lta` package contains functions implemented in Python, the code is in the [examples](https://github.com/onnovalkering/brane/tree/master/examples/lofar) directory. The `prefactor` package has been built based on a [prefactor CWL workflow](https://github.com/EOSC-LOFAR/prefactor-cwl).

## Preparation
We demonstrate the implementation using [Bakery](/brane/bakery) and JupyterLab. Starting by bringing the required packages into scope, and creating a variable that holds the identifier of a LTA observation.

```go
import "fs"
import "lta"
import "prefactor"

observation := 246403
```

The `lta` package requires secrets: <abbr title="Long-term archive">LTA</abbr> credentials and a GRID certificate proxy. 

We add these to Brane's vault:

<video
    id="brane-clip-1"
    class="video-js"
    controls
    preload="auto"
    width="736"
    height="415"
    data-setup="{}"
>
<source src="/brane/assets/video/brane-clip-1.mp4" type="video/mp4" />
<p class="vjs-no-js">
    To view this video please enable JavaScript, and consider upgrading to a web browser that
    <a href="https://videojs.com/html5-video-support/" target="_blank">supports HTML5 video</a>.
</p>
</video>

## Notebook
For each step of the pipeline, we execute a chunk of Bakery code.

### Staging LTA files
We perform the first step of the pipeline with the following code:

```go
staging := stage observation files
wait until staging = "success"
```

<video
    id="brane-clip-1"
    class="video-js"
    controls
    preload="auto"
    width="736"
    height="415"
    data-setup="{}"
>
<source src="/brane/assets/video/brane-clip-2.mp4" type="video/mp4" />
<p class="vjs-no-js">
    To view this video please enable JavaScript, and consider upgrading to a web browser that
    <a href="https://videojs.com/html5-video-support/" target="_blank">supports HTML5 video</a>.
</p>
</video>

### Downloading LTA files
After the files have been staged, we can proceed to the download:

```go
dir := new_temp_directory
download observation files to dir
```

<video
    id="brane-clip-1"
    class="video-js"
    controls
    preload="auto"
    width="736"
    height="415"
    data-setup="{}"
>
<source src="/brane/assets/video/brane-clip-3.mp4" type="video/mp4" />
<p class="vjs-no-js">
    To view this video please enable JavaScript, and consider upgrading to a web browser that
    <a href="https://videojs.com/html5-video-support/" target="_blank">supports HTML5 video</a>.
</p>
</video>


## Running calibration tasks
Coming soon.

## Displaying output
Coming soon.

## Conclusion
Coming soon.

```go
import "fs"
import "lta"
import "prefactor"

observation := ?? as Integer

staging := stage observation files
wait until staging status = "success"

directory := new_directory
download observation files to directory

archives := "*.tar" in directory
measuresets := extract archives to directory

input := CalibrateInput {
    ms_array: measuresets,
    
    // Parameters
    cellsize_lowres_deg: 0.00694,
    image_padding: 1.4,
    maxlambda_lowres: 2000
    reference_station: "CS001HBA0",
}

result := calibrate input
```

<script src="https://vjs.zencdn.net/7.8.4/video.js"></script>
