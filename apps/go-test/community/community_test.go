package community

import (
	"encoding/json"
	"fmt"
	"github.com/stretchr/testify/assert"
	"sdkTest/lib"
	"testing"
)

const (
	aclApp = "acl_app"
	//tokenApp = "token_app"
	votingApp = "voting_app"
	systemApp = "system_app"
	aclAppAddress    = "0x3422482938473294324238204824323327492323"
	tokenAppAddress  = "0x0897482938473294324238204824323327492397"
	votingAppAddress = "0x3474953938473294324238204824323327492323"

	systemAppAddress = "0x7834953938419294324238204824323327492323"
)


func TestQueryApps(t *testing.T) {
	//
	instance := lib.LoadContract("../../community/target/community.wasm")
	res := instance.Invoke([]interface{}{"init_contract", aclAppAddress, votingAppAddress, tokenAppAddress})
	assert.Equal(t, res, "Success")

	res = instance.Invoke([]interface{}{"query_apps"})
	toAppsMap(res)

	res = instance.Invoke([]interface{}{"query_app", aclApp})
	fmt.Printf("query result: %s\n", res)
}

func TestAddApp(t *testing.T) {
	//
	instance := lib.LoadContract("../../community/target/community.wasm")
	res := instance.Invoke([]interface{}{aclAppAddress, votingAppAddress, tokenAppAddress})
	assert.Equal(t, res, "Success")

	res = instance.Invoke([]interface{}{"add_app", systemApp, systemAppAddress})
	assert.Equal(t, res, "Success")

	res = instance.Invoke([]interface{}{"query_apps"})
	toAppsMap(res)
}


func TestRemoveApp(t *testing.T) {
	//
	instance := lib.LoadContract("../../community/target/community.wasm")
	res := instance.Invoke([]interface{}{aclAppAddress, votingAppAddress, tokenAppAddress})
	assert.Equal(t, res, "Success")

	res = instance.Invoke([]interface{}{"remove_app", votingApp})
	assert.Equal(t, res, "Success")

	res = instance.Invoke([]interface{}{"query_apps"})
	toAppsMap(res)
}

func TestQueryBalances(t *testing.T) {
	//
	instance := lib.LoadContract("../../community/target/community.wasm")
	res := instance.Invoke([]interface{}{aclAppAddress, votingAppAddress, tokenAppAddress})
	assert.Equal(t, res, "Success")

	res = instance.Invoke([]interface{}{"query_balances"})
	fmt.Println(res)
}

func TestQueryVotes(t *testing.T) {
	//
	instance := lib.LoadContract("../../community/target/community.wasm")
	res := instance.Invoke([]interface{}{aclAppAddress, votingAppAddress, tokenAppAddress})
	assert.Equal(t, res, "Success")

	res = instance.Invoke([]interface{}{"query_votes"})
	fmt.Println(res)
}

func TestQueryPermissions(t *testing.T) {
	//
	instance := lib.LoadContract("../../community/target/community.wasm")
	res := instance.Invoke([]interface{}{aclAppAddress, votingAppAddress, tokenAppAddress})
	assert.Equal(t, res, "Success")

	res = instance.Invoke([]interface{}{"query_permissions"})
	fmt.Println(res)
}

func TestQueryDefaultPermissions(t *testing.T) {
	instance := lib.LoadContract("../../community/target/community.wasm")
	res := instance.Invoke([]interface{}{"init", aclAppAddress, votingAppAddress, tokenAppAddress})
	assert.Equal(t, res, "Success")

	res = instance.Invoke([]interface{}{"default_permission"})
	fmt.Println(res)
}

func toAppsMap(res string) {
	var appMap map[string]string
	err := json.Unmarshal([]byte(res), &appMap)
	if err != nil {
		panic(err)
	}else {
		fmt.Println(appMap)
	}
}