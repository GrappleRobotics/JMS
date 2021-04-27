package util

import (
	"fmt"

	log "github.com/sirupsen/logrus"
)

func ElideError(anything interface{}, e error) interface{} {
	if e != nil {
		log.Fatalf("Error elided: %s", e)
	}
	return anything
}

func Wrap(err error, context string, args ...interface{}) error {
	if err != nil {
		context = fmt.Sprintf(context, args...)
		return fmt.Errorf("%s: %v", context, err)
	}
	return nil
}
