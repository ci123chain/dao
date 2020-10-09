package lib

import (
	"fmt"
	wasm "github.com/wasmerio/go-ext-wasm/wasmer"
)

type ExportType func(...interface{}) (wasm.Value, error)

type Instance struct {
	instance *wasm.Instance
	invokeM ExportType
	initM ExportType
}

func NewInstance(instance *wasm.Instance) *Instance {
	invoke, exist := instance.Exports["invoke"]
	if !exist {
		return nil
	}
	init, exist := instance.Exports["init"]
	if !exist {
		return nil
	}
	return &Instance{
		instance: instance,
		invokeM: invoke,
		initM: init,
	}
}

var ch chan string

func (ins *Instance) Invoke(param []interface{}) string {
	ch = make(chan string, 1)

	fmt.Printf("\n==============================\ncall %s\n", param[0])
	inputData[InputDataTypeParam] = serialize(param)

	defer func() {
		if err := recover(); err != nil {
			fmt.Println(err)
		}
	}()
	_, err := ins.invokeM()
	if err != nil {
		panic(err)
	}
	select {
	case res := <-ch:
		return res
	default:
		return ""
	}

}

func (ins *Instance) Init(param []interface{}) {
	fmt.Printf("\n==============================\ncall %s\n", "init")
	inputData[InputDataTypeParam] = serialize(param)

	defer func() {
		if err := recover(); err != nil {
			fmt.Println(err)
		}
	}()
	_, err := ins.initM()
	if err != nil {
		panic(err)
	}
}

func (ins *Instance)Close() {
	ins.instance.Close()
}