PFSense 2.7.0
2 cores, 2048MB ram
vmbr9000, vmbr9999

VLAN Configuration? y
VLAN Capable Interface? vtnet1
VLAN tag? 100

WAN interface? vtnet0
LAN interface? vtnet1.100
OPT1 interface? nothing

WAN will timeout because no DHCP, just wait a minute and it'll stop trying.

Go to Set interface(s) IP address and use the following settings for LAN:
- Configure via DHCP? No
- IP Address: 10.0.100.1
- Subnet bit count: 24
- Hit enter for none
- IPv6? No
- DHCP server on LAN? yes
- Start of range: 10.0.100.100
- End of range: 10.0.100.254
- Revert to HTTP as webConfigurator protocol? yes


Now we can configure from the admin network. Browse to 10.0.100.1 in a browser once connected to VLAN 100 from a regular computer. Default creds are admin/pfsense.

![[Pasted image 20230812141049.png]]

![[Pasted image 20230812141105.png]]

Leave WAN as default (blank)

![[Pasted image 20230812141219.png]]

Set an admin password. Usually this will be `jmsR0cks`

# Configuring from Scratch

## Interfaces > WAN

![[Pasted image 20230812142245.png]]

## Interfaces > LAN
Rename to ADMIN
![[Pasted image 20230812141530.png]]

# System > Interfaces > VLANs
Create a VLAN for each 10, 20, 30, 40, 50, 60 on your trunk interface (usually vtnet1)

![[Pasted image 20230812151755.png]]

# System > Advanced > Admin Access

![[Pasted image 20230812143932.png]]

