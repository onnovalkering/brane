# Generated by the gRPC Python protocol compiler plugin. DO NOT EDIT!
"""Client and server classes corresponding to protobuf-defined services."""
import grpc

from . import driver_pb2 as driver__pb2


class DriverServiceStub(object):
    """Missing associated documentation comment in .proto file."""

    def __init__(self, channel):
        """Constructor.

        Args:
            channel: A grpc.Channel.
        """
        self.CreateSession = channel.unary_unary(
                '/driver.DriverService/CreateSession',
                request_serializer=driver__pb2.CreateSessionRequest.SerializeToString,
                response_deserializer=driver__pb2.CreateSessionReply.FromString,
                )
        self.Execute = channel.unary_stream(
                '/driver.DriverService/Execute',
                request_serializer=driver__pb2.ExecuteRequest.SerializeToString,
                response_deserializer=driver__pb2.ExecuteReply.FromString,
                )


class DriverServiceServicer(object):
    """Missing associated documentation comment in .proto file."""

    def CreateSession(self, request, context):
        """Missing associated documentation comment in .proto file."""
        context.set_code(grpc.StatusCode.UNIMPLEMENTED)
        context.set_details('Method not implemented!')
        raise NotImplementedError('Method not implemented!')

    def Execute(self, request, context):
        """Missing associated documentation comment in .proto file."""
        context.set_code(grpc.StatusCode.UNIMPLEMENTED)
        context.set_details('Method not implemented!')
        raise NotImplementedError('Method not implemented!')


def add_DriverServiceServicer_to_server(servicer, server):
    rpc_method_handlers = {
            'CreateSession': grpc.unary_unary_rpc_method_handler(
                    servicer.CreateSession,
                    request_deserializer=driver__pb2.CreateSessionRequest.FromString,
                    response_serializer=driver__pb2.CreateSessionReply.SerializeToString,
            ),
            'Execute': grpc.unary_stream_rpc_method_handler(
                    servicer.Execute,
                    request_deserializer=driver__pb2.ExecuteRequest.FromString,
                    response_serializer=driver__pb2.ExecuteReply.SerializeToString,
            ),
    }
    generic_handler = grpc.method_handlers_generic_handler(
            'driver.DriverService', rpc_method_handlers)
    server.add_generic_rpc_handlers((generic_handler,))


 # This class is part of an EXPERIMENTAL API.
class DriverService(object):
    """Missing associated documentation comment in .proto file."""

    @staticmethod
    def CreateSession(request,
            target,
            options=(),
            channel_credentials=None,
            call_credentials=None,
            insecure=False,
            compression=None,
            wait_for_ready=None,
            timeout=None,
            metadata=None):
        return grpc.experimental.unary_unary(request, target, '/driver.DriverService/CreateSession',
            driver__pb2.CreateSessionRequest.SerializeToString,
            driver__pb2.CreateSessionReply.FromString,
            options, channel_credentials,
            insecure, call_credentials, compression, wait_for_ready, timeout, metadata)

    @staticmethod
    def Execute(request,
            target,
            options=(),
            channel_credentials=None,
            call_credentials=None,
            insecure=False,
            compression=None,
            wait_for_ready=None,
            timeout=None,
            metadata=None):
        return grpc.experimental.unary_stream(request, target, '/driver.DriverService/Execute',
            driver__pb2.ExecuteRequest.SerializeToString,
            driver__pb2.ExecuteReply.FromString,
            options, channel_credentials,
            insecure, call_credentials, compression, wait_for_ready, timeout, metadata)