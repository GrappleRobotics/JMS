package util

import log "github.com/sirupsen/logrus"

func ElideError(anything interface{}, e error) interface{} {
	if e != nil {
		log.Fatalf("Error elided: %s", e)
	}
	return anything
}
