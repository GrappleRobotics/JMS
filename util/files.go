package util

import (
	"io/ioutil"
	"path"
	"path/filepath"
	"runtime"
)

var (
	_, fname, _, _ = runtime.Caller(0)
	ModuleRootPath = filepath.Join(filepath.Dir(fname), "../")
)

func ModuleFilePath(paths ...string) string {
	paths = append([]string{ModuleRootPath}, paths...)
	return path.Join(paths...)
}

func ReadModuleFile(paths ...string) ([]byte, error) {
	return ioutil.ReadFile(ModuleFilePath(paths...))
}
