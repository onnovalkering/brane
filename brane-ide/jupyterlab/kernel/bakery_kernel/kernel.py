from ipykernel.kernelbase import Kernel
from os import environ
from zmq import Context, REQ
from json import loads, dumps
from requests import get, post
from urllib.parse import urljoin
from time import sleep

API_HOST = environ.get("API_HOST")

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
        self.session_uuid = None

        self.context = Context()
        self.repl = self.context.socket(REQ)
        self.repl.connect('tcp://localhost:5555')

    def do_execute(self, code, silent, store_history=True, user_expressions=None, allow_stdin=False):
        if self.session_uuid is None:
            self.create_session()

        self.repl.send(code.encode('UTF-8'))
        instructions = loads(self.repl.recv())

        invocation_uuid = self.create_invocation(instructions)
        status = self.get_invocation_status(invocation_uuid)
        content = {
            'data': {
                "application/json": status
            },
            'metadata': {},
            'transient': {
                'display_id': invocation_uuid
            }
        }

        self.send_response(self.iopub_socket, 'display_data', content)

        while status["invocation"]["status"] != "complete":
            sleep(1)

            status = self.get_invocation_status(invocation_uuid)
            content = {
                'data': {
                    "application/json": status
                },
                'metadata': {},
                'transient': {
                    'display_id': invocation_uuid
                }
            }

            self.send_response(self.iopub_socket, 'update_display_data', content)

        return {
            'status': 'ok',
            'execution_count': self.execution_count,
            'payload': [],
            'user_expressions': {},
        }

    def create_session(self):
        response = post(urljoin(API_HOST, "sessions"))
        content = response.json()

        self.session_uuid = content["uuid"]

    def create_invocation(self, instructions):
        payload = {
            "session": self.session_uuid,
            "instructions": instructions,
        }
        response = post(urljoin(API_HOST, "invocations"), json=payload)
        content = response.json()

        return content["uuid"]

    def get_invocation_status(self, invocation_uuid):
        response = get(urljoin(API_HOST, "invocations") + f"/{invocation_uuid}/status")
        return response.json()
