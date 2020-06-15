from ipykernel.kernelbase import Kernel
from pexpect.replwrap import REPLWrapper
from os import environ

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

        # Start a new Bakery REPL session
        environ['TERM'] = 'xterm-256color'
        self.repl = REPLWrapper('/kernel/brane -s repl', self.prompt, None)

    def do_execute(self, command, silent, store_history=True, user_expressions=None, allow_stdin=False):
        output = self.repl.run_command(command)
        output = '\n'.join(output.strip().split('\r\n')[1:]).strip()

        if output != '' :
            stream_content = {'name': 'stdout', 'text': output }
            self.send_response(self.iopub_socket, 'stream', stream_content)

        return {
            'status': 'ok',
            'execution_count': self.execution_count,
            'payload': [],
            'user_expressions': {},
        }
