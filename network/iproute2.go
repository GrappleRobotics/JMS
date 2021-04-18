package network

import (
	"github.com/vishvananda/netlink"
)

func ConfigureAdmin(a AdminNetwork) error {
	ClearInterface(a.Interface)
	// In our case, the router and server are the same machine - but with two IP addresses on the
	// admin interface. By convention, we use .1 for the router - however the DriverStation is
	// hardcoded to look at .5 for the FMS Server
	if err := addAddrToIface(a.Interface, IPtoCIDR(a.Router, a.Network)); err != nil {
		return err
	}
	if err := addAddrToIface(a.Interface, IPtoCIDR(a.Server, a.Network)); err != nil {
		return err
	}
	return nil
}

func ConfigureInterfaceForNetwork(n TeamNetwork) error {
	ClearInterface(n.Interface)
	cidr := IPtoCIDR(n.Router, n.Network)
	return addAddrToIface(n.Interface, cidr)
}

func ClearInterface(iface netlink.Link) error {
	addrs, err := netlink.AddrList(iface, netlink.FAMILY_ALL)
	if err != nil {
		return err
	}
	for _, addr := range addrs {
		if err := netlink.AddrDel(iface, &addr); err != nil {
			return err
		}
	}
	return nil
}

func addAddrToIface(iface netlink.Link, cidr string) error {
	addr, err := netlink.ParseAddr(cidr)
	if err != nil {
		return err
	}
	err = netlink.AddrAdd(iface, addr)
	return err
}
