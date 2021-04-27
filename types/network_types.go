package types

import (
	"fmt"
	"net"

	"github.com/JaciBrunning/jms/util"
)

type NetworkProvider interface {
	Up(ArenaNetwork) error
}

type TeamNetwork struct {
	Present   bool
	DS        DriverStation
	Team      int
	Network   net.IPNet
	Router    net.IP
	DHCPRange [2]net.IP
}

type AdminNetwork struct {
	Network   net.IPNet
	Router    net.IP
	Server    net.IP
	DHCPRange [2]net.IP
}

type ArenaNetwork struct {
	Admin AdminNetwork
	Blue  []TeamNetwork
	Red   []TeamNetwork
}

const NO_TEAM = 0

func NewTeamNetwork(ds DriverStation, team int) (TeamNetwork, error) {
	nw := TeamNetwork{}
	nw.DS = ds
	nw.Team = team
	nw.Present = team != NO_TEAM

	if nw.Present {
		cidr := fmt.Sprintf("%s/24", teamToIP(team, 0))
		_, cidrNet, err := net.ParseCIDR(cidr)
		if err != nil {
			return TeamNetwork{}, util.Wrap(err, "NewTeamNetwork")
		}
		if cidrNet == nil {
			return TeamNetwork{}, fmt.Errorf("NewTeamNetwork: Could not parse network from CIDR: %s", cidr)
		}

		nw.Network = *cidrNet
		nw.Router = net.ParseIP(teamToIP(team, 1)).To4()
		nw.DHCPRange[0] = net.ParseIP(teamToIP(team, 100)).To4()
		nw.DHCPRange[1] = net.ParseIP(teamToIP(team, 200)).To4()
	}

	return nw, nil
}

func NewAdminNetwork() (AdminNetwork, error) {
	nw := AdminNetwork{}

	_, n, err := net.ParseCIDR("10.0.100.0/24")
	if err != nil {
		return AdminNetwork{}, util.Wrap(err, "NewAdminNetwork")
	}
	nw.Network = *n
	nw.Router = net.ParseIP("10.0.100.1").To4()
	nw.Server = net.ParseIP("10.0.100.5").To4()
	nw.DHCPRange[0] = net.ParseIP("10.0.100.100").To4()
	nw.DHCPRange[1] = net.ParseIP("10.0.100.200").To4()

	return nw, nil
}

// TODO: New ArenaNetwork, taking some data structure from the field layout

func (n ArenaNetwork) EachTeam() []TeamNetwork {
	return append(n.Blue, n.Red...)
}

func (n TeamNetwork) Netmask() string {
	return net.IP(n.Network.Mask).String()
}

func (n AdminNetwork) Netmask() string {
	return net.IP(n.Network.Mask).String()
}

func teamToIP(team int, host int) string {
	return fmt.Sprintf("10.%d.%d.%d", team/100, team%100, host)
}
