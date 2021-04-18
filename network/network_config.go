package network

import (
	"fmt"

	"github.com/vishvananda/netlink"
)

type JMSNetworkConfig struct {
	Interfaces struct {
		WAN   string
		Admin string
		Blue  []string
		Red   []string
	}
}

func (n JMSNetworkConfig) Validate() error {
	ifaces := n.Interfaces
	all_ifaces := make(map[string]string)
	if err := validateInterface("WAN", ifaces.WAN, all_ifaces); err != nil {
		return err
	}
	if err := validateInterface("Admin", ifaces.Admin, all_ifaces); err != nil {
		return err
	}
	for i, iface := range ifaces.Blue {
		name := fmt.Sprintf("Blue[%d]", i)
		if err := validateInterface(name, iface, all_ifaces); err != nil {
			return err
		}
	}
	for i, iface := range ifaces.Red {
		name := fmt.Sprintf("Red[%d]", i)
		if err := validateInterface(name, iface, all_ifaces); err != nil {
			return err
		}
	}
	return nil
}

func validateInterface(key string, iface string, all map[string]string) error {
	if iface == "" {
		return fmt.Errorf("[NetworkConfig] iface for %s not set", key)
	}
	if !DoesInterfaceExist(iface) {
		return fmt.Errorf("[NetworkConfig] %s iface %s does not exist", key, iface)
	}
	if existingVal, exists := all[iface]; exists {
		return fmt.Errorf("[NetworkConfig] duplicate interface %s (in: %s, previously in: %s)", iface, key, existingVal)
	}
	all[iface] = key
	return nil
}

func DoesInterfaceExist(iface string) bool {
	_, err := netlink.LinkByName(iface)
	return err == nil
}
