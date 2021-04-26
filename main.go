package main

import (
	"io/ioutil"

	"github.com/JaciBrunning/jms/arena"
	"github.com/JaciBrunning/jms/network"
	"github.com/JaciBrunning/jms/util"

	"gopkg.in/yaml.v2"

	log "github.com/sirupsen/logrus"
)

type JMSConfig struct {
	Networking network.JMSNetworkConfig
}

type DHCPConfig struct {
	Network, Mask          string
	Router                 string
	RangeLower, RangeUpper string
}

func main() {
	cfg := JMSConfig{}
	cfgData, err := ioutil.ReadFile("jms-config.yml")
	if err != nil {
		log.Fatalf("Error: %v", err)
	}
	err = yaml.Unmarshal([]byte(cfgData), &cfg)
	if err != nil {
		log.Fatalf("Error: %v", err)
	}

	if err := cfg.Networking.Validate(); err != nil {
		log.Fatalf("Error: %v", err)
	}

	n := network.ArenaNetwork{}
	n.Teams = []network.TeamNetwork{
		util.ElideError(network.GenerateTeamNetwork(cfg.Networking, arena.DriverStation{"Blue", 1}, 5333)).(network.TeamNetwork),
		util.ElideError(network.GenerateTeamNetwork(cfg.Networking, arena.DriverStation{"Blue", 2}, 4788)).(network.TeamNetwork),
		util.ElideError(network.GenerateTeamNetwork(cfg.Networking, arena.DriverStation{"Blue", 3}, 5663)).(network.TeamNetwork),
		util.ElideError(network.GenerateTeamNetwork(cfg.Networking, arena.DriverStation{"Red", 1}, 3132)).(network.TeamNetwork),
		util.ElideError(network.GenerateTeamNetwork(cfg.Networking, arena.DriverStation{"Red", 2}, 4613)).(network.TeamNetwork),
		util.ElideError(network.GenerateTeamNetwork(cfg.Networking, arena.DriverStation{"Red", 3}, 5331)).(network.TeamNetwork),
	}
	n.Admin = util.ElideError(network.GenerateAdminNetwork(cfg.Networking)).(network.AdminNetwork)
	n.WAN = util.ElideError(network.GenerateWANNetwork(cfg.Networking)).(network.WANNetwork)

	err = n.Up()
	if err != nil {
		log.Fatalf("Error: %v", err)
	}
}
