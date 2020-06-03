$base: "https://w3id.org/cwl/cwl#"

$namespaces:
  s: "http://schema.org/"

s:name: 'echo'
s:description: 'Simple echo command-line tool.'
s:version: '1.0.0'

cwlVersion: v1.0
class: CommandLineTool
label: echo
baseCommand: echo

inputs:
  input:
    type: string
    inputBinding:
      position: 1

outputs:
  output:
    type: stdout
