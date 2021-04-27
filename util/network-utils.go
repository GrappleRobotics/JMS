package util

import "net"

func IPtoCIDR(ip net.IP, network net.IPNet) string {
	return (&net.IPNet{IP: ip, Mask: network.Mask}).String()
}
