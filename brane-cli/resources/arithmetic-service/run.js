#!/usr/bin/env node
const http = require('http');
const urllib = require('url');

async function requestHandler(req, res) {
    const url = urllib.parse(req.url);
    const path = url.pathname.trim()
    const body = await getBody(req);

    const a = parseInt(body['a']);
    const b = parseInt(body['b']);
    let c;

    if (path === '/add') {
        c = a + b;
    }

    if (path === '/substract') {
        c = a - b;
    }

    if (path === '/multiply') {
        c = a * b;
    }

    if (path === '/divide') {
        c = a / b;
    }

    res.statusCode = 200;
    res.end(JSON.stringify({c}));
}

function getBody(req) {
    return new Promise(function(resolve, reject) {
        let body = '';

        req.on('data', chunk => {
            body += chunk.toString();
        });
        req.on('end', () => {
            resolve(JSON.parse(body));
        });
    });
}

const port = process.env['PORT'] || 5050;
const host = process.env['HOST'] || 'localhost';

let server = http.createServer();
server.on('request', requestHandler);
server.listen(port, host);