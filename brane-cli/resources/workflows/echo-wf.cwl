$base: "https://w3id.org/cwl/cwl#"

$namespaces:
  s: "http://schema.org/"

s:name: "echo-wf"
s:description: "Simple echo workflow."
s:version: "1.0.0"

cwlVersion: v1.0
class: Workflow
label: echo-wf

inputs:
  message:
    type: string

steps:
  echo-step:
    run: echo.cwl
    in:
      message: message
    out:
      - message

outputs:
  message:
    type: File
    outputSource: echo-step/message

