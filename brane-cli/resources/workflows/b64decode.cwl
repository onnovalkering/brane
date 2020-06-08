$base: "https://w3id.org/cwl/cwl#"

$namespaces:
  s: "http://schema.org/"

s:name: 'b64decode'
s:description: 'Simple Base64 decoding tool.'
s:version: '1.0.0'

cwlVersion: v1.0
class: CommandLineTool
label: b64decode
baseCommand: "echo"
requirements:
  - class: ShellCommandRequirement

inputs:
  input:
    type: string
    inputBinding:
      position: 1
  pipe:
    type: string
    default: '| base64 -d'
    inputBinding:
      shellQuote: false
      position: 2

outputs:
  output:
    type: stdout
