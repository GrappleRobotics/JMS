# Testing for protobuf
require_relative 'protos/nodes_pb.rb'
require 'socket'

include Jms::Electronics;

PORT = 5333
HOST = 'localhost'

ROLE = NodeRole::NODE_BLUE

estops = [ false, false, false ]

s = TCPSocket.new HOST, PORT

# Send thread
t1 = Thread.new do 
  loop do
    msg = UpdateNode2Field.new(ipv4: [10, 0, 100, 5].pack('c*'), role: ROLE)
    msg["alliance"] = UpdateNode2Field::Alliance.new(
      estop1: estops[0],
      estop2: estops[1],
      estop3: estops[2]
    )

    msg = UpdateNode2Field.encode msg
    s.write msg
    sleep(0.5)
  end
end

t2 = Thread.new do
  loop do
    recv = s.recv(256)
    msg = UpdateField2Node.decode recv
    puts "-> #{msg.inspect}"
  end
end

go = true

while go do
  puts "What do you want to do?"
  puts "EX: Estop station X, FX: Un-Estop station X"
  msg = gets
  if msg[0] == 'E'
    stn = msg[1].to_i
    estops[stn - 1] = true
    puts estops.inspect
  elsif msg[0] == 'F'
    stn = msg[1].to_i
    estops[stn - 1] = false
    puts estops.inspect
  end
end

t1.join
t2.join