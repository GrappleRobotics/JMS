package network_onboard

import (
	"github.com/JaciBrunning/jms/util"
	"github.com/vishvananda/netlink"

	log "github.com/sirupsen/logrus"
)

func configureIPRoute2(iface string, addresses ...string) error {
	if !util.DANGER_ZONE_ENABLED {
		log.Warnf("IPRoute2 Configuration skipped for iface %s: danger zone disabled", iface)
		return nil
	}

	// Get link
	link, err := netlink.LinkByName(iface)
	if err != nil {
		return util.Wrap(err, "IPRoute2")
	}

	// Clear interface configuration
	existingAddrs, err := netlink.AddrList(link, netlink.FAMILY_ALL)
	if err != nil {
		return util.Wrap(err, "IPRoute2")
	}

	for _, addr := range existingAddrs {
		if err = netlink.AddrDel(link, &addr); err != nil {
			return util.Wrap(err, "IPRoute2")
		}
	}

	// Add new addresses
	for _, addr := range addresses {
		addr, err := netlink.ParseAddr(addr)
		if err != nil {
			return err
		}
		if err = netlink.AddrAdd(link, addr); err != nil {
			return util.Wrap(err, "IPRoute2")
		}
	}
	return nil
}

// func ConfigureAdmin(a types.AdminNetwork) error {
// 	if !util.DANGER_ZONE_ENABLED {
// 		logger.Warn("Admin network IP configuration skipped: danger zone disabled")
// 		return nil
// 	}

// 	clearInterface(a.Interface)
// 	// In our case, the router and server are the same machine - but with two IP addresses on the
// 	// admin interface. By convention, we use .1 for the router - however the DriverStation is
// 	// hardcoded to look at .5 for the FMS Server
// 	if err := addAddrToIface(a.Interface, IPtoCIDR(a.Router, a.Network)); err != nil {
// 		return err
// 	}
// 	if err := addAddrToIface(a.Interface, IPtoCIDR(a.Server, a.Network)); err != nil {
// 		return err
// 	}
// 	return nil
// }

// func ConfigureInterfaceForNetwork(n TeamNetwork) error {
// 	if !util.DANGER_ZONE_ENABLED {
// 		logger.Warnf("Team network %d IP could not be configured: danger zone disabled", n.Team)
// 		return nil
// 	}

// 	clearInterface(n.Interface)
// 	cidr := IPtoCIDR(n.Router, n.Network)
// 	return addAddrToIface(n.Interface, cidr)
// }

// func clearInterface(iface netlink.Link) error {
// 	if !util.DANGER_ZONE_ENABLED {
// 		logger.Warnf("Interface %s could not be cleared: danger zone disabled.", iface.Attrs().Name)
// 		return nil
// 	}
// 	addrs, err := netlink.AddrList(iface, netlink.FAMILY_ALL)
// 	if err != nil {
// 		return err
// 	}
// 	for _, addr := range addrs {
// 		if err := netlink.AddrDel(iface, &addr); err != nil {
// 			return err
// 		}
// 	}
// 	return nil
// }

// func addAddrToIface(iface netlink.Link, cidr string) error {
// 	if !util.DANGER_ZONE_ENABLED {
// 		logger.Warnf("Address %s could not be added to interface %s: danger zone disabled.", cidr, iface.Attrs().Name)
// 		return nil
// 	}
// 	addr, err := netlink.ParseAddr(cidr)
// 	if err != nil {
// 		return err
// 	}
// 	err = netlink.AddrAdd(iface, addr)
// 	return err
// }
