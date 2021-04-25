from base64 import b64encode
from grpc import insecure_channel
from imghdr import what
from io import BytesIO
from ipykernel.kernelbase import Kernel
from os import getenv
from requests import get, post
from time import sleep
from urllib.parse import urljoin

# Generated gRPC code
from .driver_pb2 import CreateSessionRequest, ExecuteRequest
from .driver_pb2_grpc import DriverServiceStub

BRANE_API_URL = getenv('BRANE_API_URL', 'brane-api:8080')
BRANE_DRV_URL = getenv('BRANE_DRV_URL', 'brane-drv:50053')

class BakeryKernel(Kernel):
    implementation = 'Bakery'
    implementation_version = '1.0.0'
    banner = 'bakery'
    language = 'no-op'
    language_version = '0.4.0'
    language_info = {
        'name': 'Bakery',
        'mimetype': 'text/plain',
        'file_extension': '.bk',
    }

    def __init__(self, **kwargs):
        Kernel.__init__(self, **kwargs)

        self.driver = DriverServiceStub(insecure_channel(BRANE_DRV_URL))
        self.session_uuid = None

    def do_execute(self, code, silent, store_history=True, user_expressions=None, allow_stdin=False):
        if not code.strip():
            return self.complete()

        self.intercept_magic(code)

        # Create session, if not already exists
        if self.session_uuid is None:
            session = self.driver.CreateSession(CreateSessionRequest())
            self.session_uuid = session.uuid

        interrupted = False
        try:
            result = self.driver.Execute(ExecuteRequest(uuid=self.session_uuid, input=code))
            if len(result.output) > 0:
                self.publish_stream('stdout', result.output)
        except KeyboardInterrupt:
            # TODO: support keyboard interrupt (like CLI REPL)
            interrupted = True

        if interrupted:
            return {'status': 'abort', 'execution_count': self.execution_count}
        else:
            return self.complete()

    def complete(self):
        # This marks the current cell as complete
        return {
            'status': 'ok',
            'execution_count': self.execution_count,
            'payload': [],
            'user_expressions': {},
        }

    def intercept_magic(self, code):
        """
        Checks for magic and invokes it. This is done before any Bakery code.
        No need to filter magic out, as it is considered a comment by Bakery.
        """
        magics = {
            'attach': self.attach,
            'display': self.display,
            'js9': self.js9,
            'session': lambda: self.publish_stream('stdout', self.session_uuid),
        }

        lines = [l[3:].strip() for l in code.split('\n') if l.startswith('//!')]
        for line in lines:
            command = line.split(' ')
            magic = magics.get(command[0])

            if magic is not None:
                magic(*command[1:])

    def attach(self, session_uuid):
        """
        Attach to an existing session. Variables will be restored, imports not.
        """
        self.session_uuid = session_uuid

    def display(self, variable):
        """
        Retreives and displays a 'File' variable.
        """
        response = get(f"todo$SESSIONS_ENDPOINT/{self.session_uuid}/files/{variable}")
        if response.headers['content-type'] == 'text/plain':
            content = {
                'data': {
                    'text/plain': response.text
                },
                'metadata': {}
            }
        else:
            with BytesIO(response.content) as b:
                image = b.read()

            image_type = what(None, image)
            image_data = b64encode(image).decode('ascii')

            content = {
                'data': {
                    f'image/{image_type}': image_data
                },
                'metadata': {}
            }

        self.send_response(self.iopub_socket, "display_data", content)

    def js9(self, variable):
        """
        Displays a 'File' variable using JS9, only FITS files are supported.
        """
        file_url = f"todo$SESSIONS_ENDPOINT/{self.session_uuid}/files/{variable}"
        content = {
            'data': {
                'image/fits': file_url
            },
            'metadata': {}
        }

        self.send_response(self.iopub_socket, "display_data", content)

    def publish_json(self, data, update):
        """
        Publishes a JSON payload on Jupyter's IOPub channel.
        Subsequent calls should be updates, to support delta rendering.
        """
        content = {
            'data': {
                'application/json': data
            },
            'metadata': {},
            'transient': {}
        }

        message_type = "update_display_data" if update else "display_data"
        self.send_response(self.iopub_socket, message_type, content)

    def publish_stream(self, stream, text):
        """
        Publishes a 'stream' message on the Jupyter IOPub channel.
        """
        content = {
            'name' : stream,
            'text' : text,
        }

        self.send_response(self.iopub_socket, 'stream', content)

    def publish_status(self, status, invocation_uuid, update):
        """
        Publishes a status payload on Jupyter's IOPub channel.
        Subsequent calls should be updates, to support delta rendering.
        """
        content = {
            'data': {
                'application/vnd.brane.invocation+json': status
            },
            'metadata': {},
            'transient': {
                'display_id': invocation_uuid
            }
        }

        message_type = "update_display_data" if update else "display_data"
        self.send_response(self.iopub_socket, message_type, content)
