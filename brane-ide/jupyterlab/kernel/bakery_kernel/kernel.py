from ipykernel.kernelbase import Kernel
from os import environ
from zmq import Context, REQ
from json import loads, dumps

context = zmq.Context()

#  Socket to talk to server
print("Connecting to service…")
socket = context.socket(zmq.REQ)
socket.connect("tcp://localhost:5555")

dsl = b"a := 1\nb:= 2"
socket.send(dsl)
message = socket.recv()
print(message)
    

class BakeryKernel(Kernel):
    implementation = 'Bakery'
    implementation_version = '1.0'
    language = 'no-op'
    language_version = '0.1'
    language_info = {
        'name': 'Bakery',
        'mimetype': 'text/plain',
        'file_extension': '.bk',
    }
    banner = 'Bakery kernel'
    prompt = 'brane> '

    def __init__(self, **kwargs):
        Kernel.__init__(self, **kwargs)
        self.context = Context()
        self.repl = self.context.socket(REQ)
        self.repl.connect('tcp://localhost:5555')

    def do_execute(self, code, silent, store_history=True, user_expressions=None, allow_stdin=False):
        socket.send(code.encode('UTF-8'))
        instructions = load(socket.recv())

        stream_content = {'name': 'stdout', 'text': dumps(instructions) }
        self.send_response(self.iopub_socket, 'stream', stream_content)

        return {
            'status': 'ok',
            'execution_count': self.execution_count,
            'payload': [],
            'user_expressions': {},
        }