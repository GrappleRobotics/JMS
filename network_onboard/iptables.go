package network_onboard

import (
	"html/template"
	"os"
	"os/exec"

	"github.com/JaciBrunning/jms/util"
	log "github.com/sirupsen/logrus"
)

func configureIPTables(arenaNet ConfigurationAwareNetwork) error {
	if !util.DANGER_ZONE_ENABLED {
		log.Warn("iptables configuration skipped: danger zone disabled.")
		return nil
	}

	f, err := os.CreateTemp(os.TempDir(), "jms-firewall-*.rules")
	if err != nil {
		return err
	}
	log.Infof("IPTables Config File: %s", f.Name())
	err = generateIPTablesRules(arenaNet, f)
	if err != nil {
		return err
	}
	err = applyIPTablesRules(f)
	return err
}

func generateIPTablesRules(arenaNet ConfigurationAwareNetwork, file *os.File) error {
	template_file_content, err := util.ReadModuleFile("service-configs", "templates", "match.firewall.rules")
	if err != nil {
		return err
	}
	t, err := template.New("jms-firewall-rules").Parse(string(template_file_content))
	if err != nil {
		return err
	}
	return t.Execute(file, arenaNet)
}

func applyIPTablesRules(file *os.File) error {
	util.AssertDangerZone()

	// Enable ipv4 forwarding
	err := exec.Command("sysctl", "-q", "net.ipv4.ip_forward=1").Run()
	if err != nil {
		return err
	}

	// Write iptables rules
	err = exec.Command("iptables-restore", file.Name()).Run()
	if err != nil {
		return err
	}
	return nil
}
