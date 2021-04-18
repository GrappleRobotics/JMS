package network

import (
	"fmt"
	"html/template"
	"os"
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

	logger.Info("[DHCP] Stopping DHCP Service...")
	_, err = conn.StopUnit("isc-dhcp-server.service", "replace", channel)
	if err != nil {
		return err
	}
	_ = <-channel // Wait for reload to be done

	// TODO: Make this an option
	// logger.Info("[DHCP] Flushing DHCP Caches...")
	// err = os.Remove("/var/lib/dhcp/dhcpd.leases")
	// if err != nil {
	// 	return err
	// }
	// err = os.Remove("/var/lib/dhcp/dhcpd.leases~")
	// if err != nil {
	// 	return err
	// }

	logger.Info("[DHCP] Starting DHCP Service...")
	_, err = conn.ReloadOrRestartUnit("isc-dhcp-server.service", "replace", channel)
	if err != nil {
		return err
	}
	_ = <-channel               // Wait for reload to be done
	time.Sleep(2 * time.Second) // Wait for the service to know what's going on

	logger.Info("[DHCP] Checking if DHCP Service is active...")
	props, err := conn.GetUnitProperties("isc-dhcp-server.service")
	if err != nil {
		return err
	}
	if props["ActiveState"] != "active" {
		return fmt.Errorf("[DHCP] DHCP service failed to start")
	}
	logger.Info("[DHCP] DHCP Service is up!")
	return nil
}
