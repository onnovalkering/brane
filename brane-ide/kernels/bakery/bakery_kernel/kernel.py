from base64 import b64encode
from grpc import insecure_channel
from io import BytesIO
from ipykernel.kernelbase import Kernel
from os import getenv, path
from requests import get, post
from time import sleep
from urllib.parse import urljoin
from filetype import image_match
from json import loads

# Generated gRPC code
from .driver_pb2 import CreateSessionRequest, ExecuteRequest
from .driver_pb2_grpc import DriverServiceStub

BRANE_DRV_URL = getenv('BRANE_DRV_URL', 'brane-drv:50053')
BRANE_DATA_DIR = getenv('BRANE_DATA_DIR', '/home/jovyan/data')

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
        """


        """
        self.current_bytecode = None

        if not code.strip():
            return self.complete()

        self.intercept_magic(code)

        # Create session, if not already exists
        if self.session_uuid is None:
            session = self.driver.CreateSession(CreateSessionRequest())
            self.session_uuid = session.uuid

        interrupted = False
        try:
            stream = self.driver.Execute(ExecuteRequest(uuid=self.session_uuid, input=code))
            for reply in stream:
                status = self.create_status_json(reply)
                self.publish_status(status, not reply.close)

                if reply.close:
                    file_output = self.try_as_file_output(reply.output)
                    self.send_response(self.iopub_socket, "display_data", file_output)

        except KeyboardInterrupt:
            # TODO: support keyboard interrupt (like CLI REPL)
            interrupted = True

        if interrupted:
            return {'status': 'abort', 'execution_count': self.execution_count}
        else:
            return self.complete()

    def complete(self):
        """
        This marks the current cell as complete
        """
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

    def create_status_json(self, reply):
        if len(reply.bytecode) > 0:
            self.current_bytecode = reply.bytecode

        return {
            "done": reply.close,
            "output": reply.output,
            "bytecode": self.current_bytecode,
        }

    def publish_status(self, status, update):
        """
        Publishes a status payload on Jupyter's IOPub channel.
        Subsequent calls should be updates, to support delta rendering.
        """
        content = {
            'data': {
                'application/vnd.brane.invocation+json': status
            },
            'metadata': {},
            'transient': {}
        }

        message_type = "update_display_data" if update else "display_data"
        self.send_response(self.iopub_socket, message_type, content)

    def try_as_file_output(self, output: str):
        """

        """
        if not output.startswith("\"/data/"):
            return None

        output = output.strip('"').replace("/data", BRANE_DATA_DIR)
        if not path.isfile(output):
            return None

        extension = path.splitext(output)[1]

        # Render as JSON, if file extension is .json
        if extension == '.json':
            try:
                with open (output, 'rb') as f:
                    json_data = loads(f.read())
            except:
                json_data = {
                    'message': 'Please check the file, it doesn\'t seems to be valid JSON.'
                }

            return {
                'data': {
                    'application/json': json_data
                },
                'metadata': {}
            }

        # Render as HTML, if file extension is .html
        extension = path.splitext(output)[1]
        if extension == '.html':
            with open (output, 'r') as f:
                html_data = f.read()

            return {
                'data': {
                    'text/html': html_data
                },
                'metadata': {}
            }

        kind = image_match(output)
        if kind is not None:
            with open (output, 'rb') as f:
                image_data = b64encode(f.read()).decode('ascii')

            return {
                'data': {
                    kind.mime: image_data
                },
                'metadata': {}
            }
