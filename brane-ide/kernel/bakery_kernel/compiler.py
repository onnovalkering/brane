from tempfile import mkdtemp
from os import path, remove
from zmq import Context, REQ
from subprocess import Popen
from json import loads, dumps

class BakeryCompiler:
    """
    This class wraps the Bakery compile-only service provided by the Brane CLI.
    Communication with the service is done using ZeroMQ (IPC, REQ/REP).
    """

    def __init__(self):
        self.address = path.join(mkdtemp(), 'zmq')
        self.service = Popen(["brane-cli", "-d", "-s", "repl", "-c", self.address])

        self.context = Context()        
        self.socket = self.context.socket(REQ)
        self.socket.connect(f'ipc://{self.address}')

    def __del__(self):
        if hasattr(self, 'socket'):
            self.socket.close()

        if hasattr(self, 'service'):
            self.service.kill()

        if hasattr(self, 'address'):
            remove(self.address)
            remove(path.dirname(self.address))

    def compile(self, code):
        message = dumps({
            "t": "code",
            "c": code
        })
        self.socket.send_string(message)
        result = self.socket.recv()

        return loads(result)

    def inject_variables(self, variables):
        message = dumps({
            "t": "variables",
            "c": variables,
        })
        self.socket.send_string(message)
        _ = self.socket.recv()
