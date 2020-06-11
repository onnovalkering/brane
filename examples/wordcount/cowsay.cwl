$base: "https://w3id.org/cwl/cwl#"

$namespaces:
  s: "http://schema.org/"

s:name: 'cowsay'
s:version: '1.0.0'

cwlVersion: v1.0
class: CommandLineTool
label: cowsay
baseCommand: /usr/games/cowsay
hints:
  DockerRequirement:
    dockerPull: chuanwen/cowsay

inputs:
  input:
    type: string
    inputBinding:
      position: 1

outputs:
  output:
    type: stdout
