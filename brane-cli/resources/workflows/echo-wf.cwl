$base: "https://w3id.org/cwl/cwl#"

$namespaces:
  s: "http://schema.org/"

s:name: "echo"
s:description: "Simple echo workflow."
s:version: "1.0.0"

cwlVersion: v1.0
class: Workflow
label: echo-wf

inputs:
  input:
    type: string

steps:
  echo-step:
    run: echo.cwl
    in:
      input: input
    out:
      - output

outputs:
  output:
    type: File
    outputSource: echo-step/output

