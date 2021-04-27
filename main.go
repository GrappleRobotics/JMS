package main

import (
	"io/ioutil"

	"github.com/JaciBrunning/jms/network_onboard"
	"github.com/JaciBrunning/jms/types"
	"github.com/JaciBrunning/jms/util"

	"gopkg.in/yaml.v2"

	log "github.com/sirupsen/logrus"
)

type JMSConfig struct {
	Networking network_onboard.OnboardNetworkConfig
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

	n := types.ArenaNetwork{}
	n.Blue = []types.TeamNetwork{
		util.ElideError(types.NewTeamNetwork(types.DriverStation{"Blue", 1}, 5333)).(types.TeamNetwork),
		util.ElideError(types.NewTeamNetwork(types.DriverStation{"Blue", 2}, 5663)).(types.TeamNetwork),
		util.ElideError(types.NewTeamNetwork(types.DriverStation{"Blue", 3}, 4788)).(types.TeamNetwork),
	}
	n.Red = []types.TeamNetwork{
		util.ElideError(types.NewTeamNetwork(types.DriverStation{"Red", 1}, 4613)).(types.TeamNetwork),
		util.ElideError(types.NewTeamNetwork(types.DriverStation{"Red", 2}, 0)).(types.TeamNetwork),
		util.ElideError(types.NewTeamNetwork(types.DriverStation{"Red", 3}, 6510)).(types.TeamNetwork),
	}
	n.Admin = util.ElideError(types.NewAdminNetwork()).(types.AdminNetwork)

	var net types.NetworkProvider
	net = network_onboard.New(cfg.Networking)
	err = net.Up(n)
	if err != nil {
		log.Fatalf("Error: %v", err)
	}
}
