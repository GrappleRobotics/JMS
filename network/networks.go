package network

import (
	"fmt"
	"net"
	"os"

	"github.com/JaciBrunning/jms/arena"

	log "github.com/sirupsen/logrus"
	"github.com/vishvananda/netlink"
)

type TeamNetwork struct {
	DS        arena.DriverStation
	Team      int
	Interface netlink.Link
	Network   net.IPNet
	Router    net.IP
	DHCPRange [2]net.IP
}

type AdminNetwork struct {
	Interface netlink.Link
	Network   net.IPNet
	Router    net.IP
	Server    net.IP
	DHCPRange [2]net.IP
}

type WANNetwork struct {
	Interface netlink.Link
}

type ArenaNetwork struct {
	WAN   WANNetwork
	Admin AdminNetwork
	Teams []TeamNetwork
}

// TODO: Generify iproute2 to allow for other network plugins - think dedicated
// routers, dhcp, etc for other field setups. Could potentially run on windows.

// For some reason, IPMask.String() puts it in hex form, but we want dotted form.
// Add it in here so it can be accessible from templates without a funcmap
func (n TeamNetwork) Netmask() string {
	return net.IP(n.Network.Mask).String()
}

func (n AdminNetwork) Netmask() string {
	return net.IP(n.Network.Mask).String()
}

func (n TeamNetwork) String() string {
	return fmt.Sprintf("NET[%s%d:%s (Team %d) - %s]", n.DS.Colour, n.DS.Position, n.Interface.Attrs().Name, n.Team, n.Network.String())
}

func (n AdminNetwork) String() string {
	return fmt.Sprintf("NET[ADMIN:%s - %s]", n.Interface.Attrs().Name, n.Network.String())
}

func (n WANNetwork) String() string {
	return fmt.Sprintf("NET[WAN:%s]", n.Interface.Attrs().Name)
}

func teamToIP(team int, host int) string {
	return fmt.Sprintf("10.%d.%d.%d", team/100, team%100, host)
}

func IPtoCIDR(ip net.IP, network net.IPNet) string {
	return (&net.IPNet{IP: ip, Mask: network.Mask}).String()
}

func GenerateTeamNetwork(cfg JMSNetworkConfig, ds arena.DriverStation, team int) (TeamNetwork, error) {
	robotNet := TeamNetwork{DS: ds, Team: team}
	if len(cfg.Interfaces.Blue) <= (ds.Position - 1) {
		return robotNet, fmt.Errorf("DS Position %d out of range", ds.Position)
	}

	var err error
	if ds.Colour == "Blue" {
		robotNet.Interface, err = netlink.LinkByName(cfg.Interfaces.Blue[ds.Position-1])
	} else {
		robotNet.Interface, err = netlink.LinkByName(cfg.Interfaces.Red[ds.Position-1])
	}

	if err != nil {
		return robotNet, err
	}

	cidr := fmt.Sprintf("%s/24", teamToIP(team, 0))
	_, cidrNet, err := net.ParseCIDR(cidr)
	if err != nil {
		return robotNet, err
	}
	if cidrNet == nil {
		return robotNet, fmt.Errorf("Could not parse network from CIDR %s", cidr)
	}

	robotNet.Network = *cidrNet
	robotNet.Router = net.ParseIP(teamToIP(team, 1)).To4()
	robotNet.DHCPRange[0] = net.ParseIP(teamToIP(team, 100)).To4()
	robotNet.DHCPRange[1] = net.ParseIP(teamToIP(team, 200)).To4()

	return robotNet, nil
}

func GenerateAdminNetwork(cfg JMSNetworkConfig) (AdminNetwork, error) {
	adminNet := AdminNetwork{}
	var err error

	adminNet.Interface, err = netlink.LinkByName(cfg.Interfaces.Admin)
	if err != nil {
		return adminNet, err
	}

	_, n, err := net.ParseCIDR("10.0.100.0/24")
	if err != nil {
		return adminNet, err
	}
	adminNet.Network = *n
	adminNet.Router = net.ParseIP("10.0.100.1").To4()
	adminNet.Server = net.ParseIP("10.0.100.5").To4()
	adminNet.DHCPRange[0] = net.ParseIP("10.0.100.100").To4()
	adminNet.DHCPRange[1] = net.ParseIP("10.0.100.200").To4()

	return adminNet, nil
}

func GenerateWANNetwork(cfg JMSNetworkConfig) (WANNetwork, error) {
	wanNet := WANNetwork{}
	var err error
	wanNet.Interface, err = netlink.LinkByName(cfg.Interfaces.WAN)
	if err != nil {
		return wanNet, err
	}
	return wanNet, nil
}

func (anw AdminNetwork) up() error {
	clog := log.WithField("Network", anw.String())
	clog.Debugf("%s Going up...", anw.String())
	err := ConfigureAdmin(anw)
	if err != nil {
		clog.Errorf("Error: %s", err)
		return fmt.Errorf("Error configuring iproute2 for %s: %s", anw.String(), err)
	}
	clog.Debugf("Interface UP!")
	return nil
}

func (tnw TeamNetwork) up() error {
	clog := log.WithField("Network", tnw.String())
	clog.Debugf("%s Going up...", tnw.String())
	err := ConfigureInterfaceForNetwork(tnw)
	if err != nil {
		clog.Errorf("Error: %s", err)
		return fmt.Errorf("Error configuring iproute2 for %s: %s", tnw.String(), err)
	}
	clog.Debugf("Interface UP!")
	return nil
}

func (nws ArenaNetwork) Up() error {
	clog := log.WithField("Network", "Arena")
	clog.Info("Bringing Arena Network UP")
	// Setup Admin
	err := nws.Admin.up()
	if err != nil {
		return err
	}
	// Setup Teams
	for _, tnw := range nws.Teams {
		err := tnw.up()
		if err != nil {
			return err
		}
	}
	// Setup DHCPD
	clog.Info("Configuring Arena DHCP...")
	dhcpd_file, err := os.Create("/etc/dhcp/jms-dhcp.conf")
	if err != nil {
		return err
	}
	err = GenerateDHCPDConfig(nws, dhcpd_file)
	if err != nil {
		return err
	}
	err = ReloadDHCPDService()
	if err != nil {
		return err
	}
	// Setup iptables
	clog.Info("Configuring Arena Firewall...")
	f, err := os.CreateTemp(os.TempDir(), "jms-firewall-*.rules")
	if err != nil {
		return err
	}
	err = GenerateIPTablesRules(nws, f)
	if err != nil {
		return err
	}
	err = ApplyIPTablesRules(f)
	if err != nil {
		return err
	}
	clog.Info("Arena Network IS UP")
	return nil
}
