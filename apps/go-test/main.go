package main

import (
	"sdkTest/lib"
)

func main() {
	instance := lib.LoadContract("../example/example.wasm")
	defer instance.Close()

	sendAddr := lib.NewAddress([]byte("user0000000000000000"))
	callAddr := lib.NewAddress([]byte("contract000000000000"))
	params := [][]interface{}{
		{"write_db", "time", "机器"},
		{"read_db", "time"},
		{"delete_db", "time"},
		{"send", sendAddr.ToString(), uint64(7)},
		{"get_creator"},
		{"get_invoker"},
		{"get_time"},
		{"call_contract", callAddr.ToString(), []byte{1, 2, 3}},
		{"destroy_contract"},
		//{"migrate_contract", code, "demo", "v0.0.1", "me", "email", "description"},
		{"notify"},
		{"mul", int64(1 << 60), int64(1 << 61), int64(1 << 62), int64(1 << 63 - 1)}, //overflow
		{"send", "a" + sendAddr.ToString()[1:], uint64(7)}, //panic用例
		{"read_db", "不存在的key"}, //rust panic
		{"这是一个无效的方法"},
	}

	for _, param := range params {
		instance.Invoke(param)
	}
}
