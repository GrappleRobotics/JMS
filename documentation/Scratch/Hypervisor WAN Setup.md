Instead of forwarding the WAN directly, we have to create a double-NAT situation since the WAN networks we're connecting to at the venue may be one of many IP Addresses - some of which might overlap with Team IP ranges. For this reason, we need to double-NAT so the router knows how to route traffic.

WAN (hypervisor): vmbr0 DHCP
WAN Bridge (hypervisor): vmbr9000 `172.31.71.1/24`
WAN Bridge (router): WAN `172.31.71.2/24`

![[Pasted image 20230812142147.png]]

## IPTables Setup
The following rules will ensure that all traffic coming into the WAN interface is forwarded to the JMS-Router
```
apt install iptables-persistent
iptables -t nat -A POSTROUTING -o vmbr0 -j MASQUERADE
iptables -t nat -A PREROUTING -p tcp -i vmbr9999 -j DNAT --to 172.31.71.2
iptables-save > /etc/iptables/rules.v4
```
