const http = require('http');
const responses  = require('./responses.json');

const app = http.createServer((req, res) => {
    const meta = responses[req.url];
    if (!meta) {
        console.error(`No mock respond for: ${req.url}`);

        writeNotFound(res, meta);
        return;
    }

    writeData(res, meta);
});

function writeNotFound(res) {
    res.writeHead(404);
    res.end();
}

function writeData(res, data) {
    res.writeHead(200, {'Content-Type': 'application/json'});
    res.write(JSON.stringify(data));
    res.end();
}

app.listen(3000, '127.0.0.1');

console.log('Registry API (mock) running on port 3000.');