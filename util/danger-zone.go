/**
 * Danger Zone allows JMS to modify system files, such as networking configuration,
 * in order to setup the field network. If Danger Zone is not enabled, the ordinary
 * configuration functions operate as a 'dry run' - suitable for development purposes.
 *
 * To be safe, even with Danger Zone disabled it is recommended to run JMS in a docker
 * container or VM when developing to avoid modifying critical host system files.
 *
 * Danger Zone can be enabled in one of two ways:
 *   - Placing the following text in /etc/jms-danger-zone (without the quotes)
 *         "I CONSENT TO JMS DESTROYING MY COMPUTER"
 *   - Setting the environment variable JMS_DANGER_ENABLED=true
 *         JMS_DANGER_ENABLED=false can also be used to disable Danger Zone when
 *         /etc/jms-danger-zone is present.
 */
package util

import (
	"io/ioutil"
	"os"
	"strings"

	log "github.com/sirupsen/logrus"
)

var (
	DANGER_ZONE_ENABLED = isDangerZone()
)

func isDangerZone() bool {
	data, err := ioutil.ReadFile("/etc/jms-danger-zone")
	danger := false
	if err == nil {
		danger = strings.TrimSpace(string(data)) == "I CONSENT TO JMS DESTROYING MY COMPUTER"
	}
	env_var := strings.TrimSpace(os.Getenv("JMS_DANGER_ENABLED"))
	if env_var != "" {
		danger = env_var == "true" || env_var == "t" || env_var == "yes" || env_var == "y" || env_var == "1"
	}
	if danger {
		log.Warn("\033[31m======!!!!======= DANGER ZONE ENABLED ======!!!!=======")
		log.Warn("\033[31m| JMS is in production mode. JMS will override system |")
		log.Warn("\033[31m|     configurations to setup the field network.      |")
		log.Warn("\033[31m|  If this is not what you intend, stop JMS now and   |")
		log.Warn("\033[31m|        delete the /etc/jms-danger-zone file.        |")
		log.Warn("\033[31m================= DANGER ZONE ENABLED =================")
	} else {
		log.Info("\033[32m/etc/jms-danger-zone file not present. Running in development mode, no lasting configuration changes will be made.")
	}
	return danger
}

/**
 * Only to be used in internal methods where a DANGER_ZONE_ENABLED check has already
 * been performed in the callier - in case things get called in a weird order.
 */
func AssertDangerZone() {
	if !DANGER_ZONE_ENABLED {
		log.Panic("Assertion failed: Not in the danger zone!")
	}
}
