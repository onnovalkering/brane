from ipykernel.kernelbase import Kernel
from os import environ
from zmq import Context, REQ
from json import loads, dumps
from requests import get, post
from urllib.parse import urljoin
from time import sleep

from .compiler import BakeryCompiler


API_HOST = environ.get("API_HOST")
INVOCATIONS_ENDPOINT = urljoin(API_HOST, "invocations")
SESSIONS_ENDPOINT = urljoin(API_HOST, "sessions")


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

    def __init__(self, **kwargs):
        Kernel.__init__(self, **kwargs)

        self.session = None
        self.bakery = BakeryCompiler()

    def do_execute(self, code, silent, store_history=True, user_expressions=None, allow_stdin=False):
        if self.session is None:
            self.set_session(create_session())

        instructions = self.bakery.compile(code)
        invocation = create_invocation(instructions, self.session)

        # Keep polling until invocation is complete.
        counter = 1
        while True:
            sleep(min(counter * .5, 5))

            status = self.get_invocation_status(invocation)
            self.publish_status(status, invocation, update: counter > 1)
            counter += 1

            if status["invocation"]["status"] == "complete":
                break

        return {
            'status': 'ok',
            'execution_count': self.execution_count,
            'payload': [],
            'user_expressions': {},
        }

    def set_session(self, session_uuid):
        """
        Attach to an existing session by marking it as active.
        """
        self.session_uuid = session_uuid

    def publish_status(self, status, invocation_uuid, update):
        """
        Publishes a 'display_data' message over the Jupyter IOPub channel.
        Subsequent calls should be updates, to support delta rendering.
        """
        content = {
            'data': {
                "application/json": status
            },
            'metadata': {},
            'transient': {
                'display_id': invocation_uuid
            }
        }

        message_type = "update_display_data" if update else "display_data"
        self.send_response(self.iopub_socket, message_type, content)


def create_invocation(instructions, session_uuid):
    """
    Creates a new invocation, in the context of the active session, with the provided instructions.
    """
    payload = {
        "session": session_uuid,
        "instructions": instructions,
    }
    response = post(INVOCATIONS_ENDPOINT), json=payload)
    content = response.json()

    return content["uuid"]


def create_session(self):
    """
    Creates a new session, and marks it as the active session.
    """
    response = post(SESSIONS_ENDPOINT)
    content = response.json()

    return content["uuid"]


def get_invocation_status(invocation):
    """
    Retreives the status of an invocation, can be used directly by the renderer widget.
    """
    invocation_uuid = invocation["uuid"]

    response = get(f"{INVOCATIONS_ENDPOINT}/{invocation_uuid}/status")
    return response.json()

