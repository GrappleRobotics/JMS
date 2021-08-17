# Testing for protobuf
require_relative 'protos/nodes_pb.rb'
require 'socket'

include Jms::Electronics;

PORT = 5333
HOST = 'localhost'

ROLE = NodeRole::NODE_BLUE
DATA_TYPE = :alliance
DATA = UpdateNode2Field::Alliance.new(estop1: false, estop2: false, estop3: false)

s = TCPSocket.new HOST, PORT

# Send thread
t1 = Thread.new do 
  loop do
    msg = UpdateNode2Field.new(ipv4: [10, 0, 100, 5].pack('c*'), role: ROLE)
    msg[DATA_TYPE.to_s] = DATA

    msg = UpdateNode2Field.encode msg
    s.write msg
    sleep(0.5)
  end
end

t2 = Thread.new do
  loop do
    recv = UpdateNode2Field.decode s.recv(128)
    puts recv.inspect
  end
end

t1.join
t2.join