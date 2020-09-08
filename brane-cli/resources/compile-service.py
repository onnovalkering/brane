#!/usr/bin/env python3
import zmq

context = zmq.Context()

#  Socket to talk to server
print("Connecting to service…")
socket = context.socket(zmq.REQ)
socket.connect("tcp://localhost:5555")

dsl = b"a := 1\nb:= 2"
socket.send(dsl)
message = socket.recv()
print(message)
    