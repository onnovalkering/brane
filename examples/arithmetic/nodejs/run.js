#!/usr/bin/env node
const argv = process.argv.slice(2);
const yaml = require('js-yaml');
const env = process.env;

function add(a, b) {
  return a + b;
}

function substract(a, b) {
  return a - b;
}

function multiply(a, b) {
  return a * b;
}

function divide(a, b) {
  return Math.floor(a / b);
}

const functions = {
  "add": add,
  "substract": substract,
  "multiply": multiply,
  "divide": divide,
}

const operation = argv[0];
const a = Number(env["A"]);
const b = Number(env["B"]);

const output = functions[operation](a, b)
console.log(yaml.dump({"c": output}))