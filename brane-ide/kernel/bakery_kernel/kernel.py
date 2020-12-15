from ipykernel.kernelbase import Kernel
from os import getenv
from requests import get, post
from urllib.parse import urljoin
from time import sleep
from base64 import b64encode
from imghdr import what
from io import BytesIO

from .compiler import BakeryCompiler


API_HOST = getenv('API_HOST', 'brane-api:8080')
INVOCATIONS_ENDPOINT = urljoin(f'http://{API_HOST}', 'invocations')
SESSIONS_ENDPOINT = urljoin(f'http://{API_HOST}', 'sessions')


class BakeryKernel(Kernel):
    implementation = 'Bakery'
    implementation_version = '1.0.0'
    banner = 'bakery'
    language = 'no-op'
    language_version = '0.2.0'
    language_info = {
        'name': 'Bakery',
        'mimetype': 'text/plain',
        'file_extension': '.bk',
    }

    def __init__(self, **kwargs):
        Kernel.__init__(self, **kwargs)

        self.session_uuid = None
        self.bakery = BakeryCompiler()

    def do_execute(self, code, silent, store_history=True, user_expressions=None, allow_stdin=False):
        self.intercept_magic(code)

        if self.session_uuid is None:
            self.session_uuid = create_session()

        result = self.bakery.compile(code)
        if result["variant"] == "ok" and len(result['content']) > 0:
            instructions = result["content"]
            invocation_uuid = create_invocation(instructions, self.session_uuid)

            self.poll_invocation(invocation_uuid)
        else:
            self.publish_stream("stderr", result['content'])

        return self.complete()

    def complete(self):
        # This marks the current cell as complete
        return {
            'status': 'ok',
            'execution_count': self.execution_count,
            'payload': [],
            'user_expressions': {},
        }

    def poll_invocation(self, invocation_uuid):
        # Keep polling until invocation is complete.
        counter = 1
        while True:
            sleep(min(counter * .5, 5))

            status = get_invocation_status(invocation_uuid)
            self.publish_status(status, invocation_uuid, update=counter > 1)
            counter += 1

            if status["invocation"]["status"] == "complete":
                break

    def intercept_magic(self, code):
        """
        Checks for magic and invokes it. This is done before any Bakery code.
        No need to filter magic out, as it is considered a comment by Bakery.
        """
        magics = {
            'attach': self.attach,
            'display': self.display,
            'session': lambda: self.publish_stream('stdout', self.session_uuid),
            'variables': lambda: self.publish_json(get_session_variables(self.session_uuid), False)
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

        variables = get_session_variables(session_uuid)
        variables = {v["name"]: v["type_"] for v in variables}

        self.bakery.inject_variables(variables)

        invocation_uuid = get_active_invocation(session_uuid)
        if invocation_uuid is not None:
            self.poll_invocation(invocation_uuid)

    def display(self, variable):
        """
        Retreives and displays a 'File' variable.
        """
        response = get(f"{SESSIONS_ENDPOINT}/{self.session_uuid}/files/{variable}")
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
                'application/vnd.brane.status+json': status
            },
            'metadata': {},
            'transient': {
                'display_id': invocation_uuid
            }
        }

        message_type = "update_display_data" if update else "display_data"
        self.send_response(self.iopub_socket, message_type, content)


def get_active_invocation(session_uuid):
    """
    Retreives the current active invocation, if any, of the active session.
    """
    response = get(f"{SESSIONS_ENDPOINT}/{session_uuid}/invocations?status=active")
    content = response.json()

    if len(content) == 0:
        return None
    else:
        return content[0]["uuid"]


def create_invocation(instructions, session_uuid):
    """
    Creates a new invocation, in the context of the active session, with the provided instructions.
    """
    payload = {
        "session": session_uuid,
        "instructions": instructions,
    }
    response = post(INVOCATIONS_ENDPOINT, json=payload)
    content = response.json()

    return content["uuid"]


def create_session():
    """
    Creates a new session, and marks it as the active session.
    """
    response = post(SESSIONS_ENDPOINT, json={})
    content = response.json()

    return content["uuid"]


def get_invocation_status(invocation_uuid):
    """
    Retreives the status of an invocation, can be used directly by the renderer widget.
    """
    return get(f"{INVOCATIONS_ENDPOINT}/{invocation_uuid}/status").json()


def get_session_variables(session_uuid):
    """
    Retreives the current variables that the session holds.
    """
    return get(f'{SESSIONS_ENDPOINT}/{session_uuid}/variables').json()
