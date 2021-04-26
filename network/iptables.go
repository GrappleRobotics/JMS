package network

import (
	"html/template"
	"os"
	"os/exec"
	"syscall"

	"github.com/JaciBrunning/jms/util"
)

func ConfigureIPTables(arenaNet ArenaNetwork) error {
	if !util.DANGER_ZONE_ENABLED {
		logger.Warn("iptables configuration skipped: danger zone disabled.")
		return nil
	}

	f, err := os.CreateTemp(os.TempDir(), "jms-firewall-*.rules")
	if err != nil {
		return err
	}
	err = generateIPTablesRules(arenaNet, f)
	if err != nil {
		return err
	}
	err = applyIPTablesRules(f)
	return err
}

func generateIPTablesRules(arenaNet ArenaNetwork, file *os.File) error {
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
	if !util.DANGER_ZONE_ENABLED {
		logger.Warn("iptables routing could not be applied: danger zone disabled.")
		return nil
	}
	env := os.Environ()
	// Enable ipv4 forwarding
	sysctl, err := exec.LookPath("sysctl")
	if err != nil {
		return err
	}

	args := []string{"sysctl", "-q", "net.ipv4.ip_forward=1"}
	err = syscall.Exec(sysctl, args, env)
	if err != nil {
		return err
	}

	// Write iptables rules
	iptables_restore, err := exec.LookPath("iptables-restore")
	if err != nil {
		return err
	}

	args = []string{"iptables-restore", file.Name()}
	err = syscall.Exec(iptables_restore, args, env)
	if err != nil {
		return err
	}
	return nil
}
