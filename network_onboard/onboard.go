package network_onboard

import (
	"github.com/JaciBrunning/jms/types"
	"github.com/JaciBrunning/jms/util"
	log "github.com/sirupsen/logrus"
)

type OnboardNetworkConfig struct {
	Interfaces struct {
		WAN   string
		Admin string
		Blue  []string
		Red   []string
	}
}

type OnboardNetwork struct {
	logger *log.Entry
	config OnboardNetworkConfig
}

type ConfigurationAwareNetwork struct {
	Net types.ArenaNetwork
	Cfg OnboardNetworkConfig
}

func New(config OnboardNetworkConfig) OnboardNetwork {
	return OnboardNetwork{
		logger: log.WithField("Network", "Onboard"),
		config: config,
	}
}

func (n OnboardNetwork) WithConfig(anet types.ArenaNetwork) ConfigurationAwareNetwork {
	return ConfigurationAwareNetwork{anet, n.config}
}

func (n OnboardNetworkConfig) EachTeamInterface() []string {
	return append(n.Interfaces.Blue, n.Interfaces.Red...)
}

func (n OnboardNetwork) Up(arenaNetwork types.ArenaNetwork) error {
	if !util.DANGER_ZONE_ENABLED {
		n.logger.Warn("Skipping Arena Network Configuration: Danger Zone not enabled.")
		return nil
	}

	cfgNet := n.WithConfig(arenaNetwork)

	n.logger.Info("Bringing Arena Network UP...")

	// Setup Admin Network
	n.logger.Info("Configuring Admin Network...")
	if err := n.adminUp(cfgNet.Cfg.Interfaces.Admin, arenaNetwork.Admin); err != nil {
		return util.Wrap(err, "OnboardNetwork::Up[Admin]")
	}

	// Setup Team Networks
	n.logger.Info("Configuring Team Networks...")
	for i, teamNet := range arenaNetwork.EachTeam() {
		if err := n.teamUp(cfgNet.Cfg.EachTeamInterface()[i], teamNet); err != nil {
			return util.Wrap(err, "OnboardNetwork::Up[Team=%d]", teamNet.Team)
		}
	}

	// Configure DHCP
	n.logger.Info("Configuring DHCP...")
	if err := configureDHCPD(arenaNetwork); err != nil {
		return util.Wrap(err, "OnboardNetwork::Up[DHCPD]")
	}

	// Configure Routing - IPTables
	n.logger.Info("Configuring IPTables...")
	if err := configureIPTables(cfgNet); err != nil {
		return util.Wrap(err, "OnboardNetwork::Up[IPTables]")
	}

	n.logger.Info("Arena Network is UP!")
	return nil
}

func (n OnboardNetwork) adminUp(iface string, admin types.AdminNetwork) error {
	util.AssertDangerZone()
	err := configureIPRoute2(iface,
		util.IPtoCIDR(admin.Router, admin.Network),
		util.IPtoCIDR(admin.Server, admin.Network))
	return util.Wrap(err, "AdminUp")
}

func (n OnboardNetwork) teamUp(iface string, team types.TeamNetwork) error {
	util.AssertDangerZone()
	var err error
	if team.Present {
		err = configureIPRoute2(iface, util.IPtoCIDR(team.Router, team.Network))
	} else {
		err = configureIPRoute2(iface)
	}
	return util.Wrap(err, "TeamUp")
}
