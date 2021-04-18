package network

import (
	"fmt"
	"html/template"
	"io/ioutil"
	"os"
	"strings"
	"time"

	"github.com/JaciBrunning/jms/util"
	"github.com/coreos/go-systemd/v22/dbus"

	log "github.com/sirupsen/logrus"
)

var (
	logger = log.New()
)

func GenerateDHCPDConfig(arenaNet ArenaNetwork, file *os.File) error {
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

func ReloadDHCPDService() error {
	conn, err := dbus.NewSystemdConnection()
	if err != nil {
		return err
	}
	defer conn.Close()
	channel := make(chan string)

	logger.Info("Stopping DHCP Service...")
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

	logger.Info("Starting DHCP Service...")
	_, err = conn.ReloadOrRestartUnit("isc-dhcp-server.service", "replace", channel)
	if err != nil {
		return err
	}
	_ = <-channel               // Wait for reload to be done
	time.Sleep(2 * time.Second) // Wait for the service to know what's going on

	logger.Info("Checking if DHCP Service is active...")
	props, err := conn.GetUnitProperties("isc-dhcp-server.service")
	if err != nil {
		return err
	}
	if props["ActiveState"] != "active" {
		return fmt.Errorf("DHCP service failed to start")
	}
	logger.Info("DHCP Service is up!")
	return nil
}

func maybeUpdateDHCPdConf() {
	content, err := ioutil.ReadFile("/etc/dhcp/dhcpd.conf")
	if err != nil {
		logger.Warnf("Cannot check /etc/dhcp/dhcpd.conf - %s", err)
		return
	}

	include_string := "include \"/etc/dhcp/jms-dhcp.conf\";"
	if !strings.Contains(string(content), include_string) {
		f, err := os.OpenFile("/etc/dhcp/dhcpd.conf", os.O_APPEND|os.O_WRONLY, 0644)
		if err != nil {
			logger.Warnf("Cannot append to /etc/dhcp/dhcpd.conf - %s", err)
		} else {
			fmt.Fprintf(f, "\n# Automatically added by JMS\n%s\n", include_string)
			logger.Info("JMS added to /etc/dhcp/dhcpd.conf (first-run)")
		}
	}
}
