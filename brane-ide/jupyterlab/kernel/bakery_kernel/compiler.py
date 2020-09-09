from tempfile import mkdtemp
from os import path
from zmq import Context, REQ
from subprocess import Popen

class BakeryCompiler:
    """
    This class wraps the Bakery compile-only service provided by the Brane CLI.
    Communication with the service is done using ZeroMQ (IPC, REQ/REP).
    """

    def __init__(self):
        self.address = path.join(mkdtemp(), 'zmq')
        self.service = Popen(["brane", "repl", "-c", self.address])

        self.context = Context()        
        self.socket = context.socket(REQ)
        socket.connect(f'ipc://{address}')

    def __del__(self):
        if self.service is not None:
            self.service.kill()

    def compile(self, code: bytes):
        self.socket(code.encode('UTF-8'))
        instructions = self.socket.recv()

        return loads(instructions)
