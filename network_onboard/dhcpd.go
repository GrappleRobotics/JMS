package network_onboard

import (
	"fmt"
	"html/template"
	"io/ioutil"
	"os"
	"strings"
	"time"

	"github.com/JaciBrunning/jms/types"
	"github.com/JaciBrunning/jms/util"
	"github.com/coreos/go-systemd/v22/dbus"

	log "github.com/sirupsen/logrus"
)

const JMS_DHCP_CONF_FILE = "/etc/dhcp/jms-dhcp.conf"

func configureDHCPD(arenaNet types.ArenaNetwork) error {
	if !util.DANGER_ZONE_ENABLED {
		log.Warn("DHCP Configuration Ignored: danger zone disabled.")
		return nil
	}

	dhcpd_file, err := os.Create(JMS_DHCP_CONF_FILE)
	if err != nil {
		return err
	}
	defer dhcpd_file.Close()
	err = generateDHCPDConfig(arenaNet, dhcpd_file)
	if err != nil {
		return err
	}
	err = reloadDHCPDService()
	return err
}

func generateDHCPDConfig(arenaNet types.ArenaNetwork, file *os.File) error {
	util.AssertDangerZone()
	template_file_content, err := util.ReadModuleFile("service-configs", "templates", "dhcp.conf")
	if err != nil {
		return err
	}
	t, err := template.New("jms-dhcp-config").Parse(string(template_file_content))
	if err != nil {
		return err
	}
	return t.Execute(file, arenaNet)
}

func reloadDHCPDService() error {
	util.AssertDangerZone()

	conn, err := dbus.NewSystemdConnection()
	if err != nil {
		return err
	}
	defer conn.Close()
	channel := make(chan string)

	log.Info("Stopping DHCP Service...")
	_, err = conn.StopUnit("isc-dhcp-server.service", "replace", channel)
	if err != nil {
		return err
	}
	_ = <-channel // Wait for reload to be done

	maybeUpdateDHCPdConf()

	// TODO: Make this an option. Should not be required; enabling may
	// de-lease FTA equipment. If systems contain old leases, disconnect and
	// reconnect them physically as some OSs (such as windows) require a physical
	// reconnection to send a DHCPDISCOVER instead of a DHCPREQUEST
	// logger.Info("Flushing DHCP Caches...")
	// err = os.Remove("/var/lib/dhcp/dhcpd.leases")
	// if err != nil {
	// 	return err
	// }
	// err = os.Remove("/var/lib/dhcp/dhcpd.leases~")
	// if err != nil {
	// 	return err
	// }

	log.Info("Starting DHCP Service...")
	_, err = conn.ReloadOrRestartUnit("isc-dhcp-server.service", "replace", channel)
	if err != nil {
		return err
	}
	_ = <-channel               // Wait for reload to be done
	time.Sleep(2 * time.Second) // Wait for the service to know what's going on

	log.Info("Checking if DHCP Service is active...")
	props, err := conn.GetUnitProperties("isc-dhcp-server.service")
	if err != nil {
		return err
	}
	if props["ActiveState"] != "active" {
		return fmt.Errorf("DHCP service failed to start")
	}
	log.Info("DHCP Service is up!")
	return nil
}

func maybeUpdateDHCPdConf() {
	util.AssertDangerZone()

	content, err := ioutil.ReadFile("/etc/dhcp/dhcpd.conf")
	if err != nil {
		log.Warnf("Cannot check /etc/dhcp/dhcpd.conf - %s", err)
		return
	}

	include_string := fmt.Sprintf("include \"%s\";", JMS_DHCP_CONF_FILE)
	if !strings.Contains(string(content), include_string) {
		f, err := os.OpenFile("/etc/dhcp/dhcpd.conf", os.O_APPEND|os.O_WRONLY, 0644)
		if err != nil {
			log.Warnf("Cannot append to /etc/dhcp/dhcpd.conf - %s", err)
		} else {
			fmt.Fprintf(f, "\n# Automatically added by JMS\n%s\n", include_string)
			log.Info("JMS added to /etc/dhcp/dhcpd.conf (first-run)")
		}
	}
}
