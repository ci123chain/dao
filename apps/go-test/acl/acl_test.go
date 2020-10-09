package acl

import (
	"encoding/hex"
	"fmt"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
	"sdkTest/lib"
	"testing"
)


var (
	managerAddress = "0xD1a14962627fAc768Fe885Eeb9FF072706B54c19"
	appAddress = "0x505A74675dc9C71eF3CB5DF309256952917E801e"
	appAddress2 = "0x505A74675dc9C71eF3CB5DF309256952917E803e"

	clientAddress = "0x3F43E75Aaba2c2fD6E227C10C6E7DC125A93DE3c"
	clientAddress2 = "0x3F43E75Aaba2c2fD6E227C10C6E7DC125A93DE1d"

)


func TestCreatePermissionFail(t *testing.T)  {
	instance := lib.LoadContract("../../acl/target/example.wasm")

	res := instance.Invoke([]interface{}{"create_permission", clientAddress, appAddress, "buy_book", managerAddress})
	assert.Equal(t, "you don't have permission to create", res)

	hasPermission := instance.Invoke([]interface{}{"has_permission", clientAddress, appAddress, "buy_book", managerAddress})
	assert.Equal(t, "false", hasPermission)

	hasPermission = instance.Invoke([]interface{}{"has_permission", clientAddress, appAddress, "buy_drink", managerAddress})
	assert.Equal(t, "false", hasPermission)

	hasPermission = instance.Invoke([]interface{}{"has_permission", clientAddress, appAddress2, "buy_book", managerAddress})
	assert.Equal(t, "false", hasPermission)
}


func TestCreatePermissionSuccess(t *testing.T)  {
	lib.ContractAddress = lib.NewAddress([]byte("addr1111111111111111"))
	lib.InvokerAddress = lib.NewAddress([]byte("addr2211111111111222"))
	cAddr := lib.ContractAddress.ToString()
	iAddr := lib.InvokerAddress.ToString()

	instance := lib.LoadContract("../../acl/target/example.wasm")
	instance.Init([]interface{}{iAddr})

	hasPermission := instance.Invoke([]interface{}{"has_permission", iAddr, cAddr, "acl.createperm"})
	require.Equal(t, "true", hasPermission)

	res := instance.Invoke([]interface{}{"create_permission", clientAddress, cAddr, "buy_book", managerAddress})
	require.Equal(t, "success", res)

	hasPermission = instance.Invoke([]interface{}{"has_permission", clientAddress, cAddr, "buy_book"})
	assert.Equal(t,"true", hasPermission)

	hasPermission = instance.Invoke([]interface{}{"has_permission", clientAddress, appAddress, "buy_book"})
	assert.Equal(t, "false", hasPermission)
}


func TestCreatePermissionWithOtherApp(t *testing.T)  {
	lib.ContractAddress = lib.NewAddress([]byte("addr1111111111111111"))
	communityAddr := lib.NewAddress([]byte("addr2211111111111222"))
	cAddr := lib.ContractAddress.ToString()
	commAddr := communityAddr.ToString()

	instance := lib.LoadContract("../../acl/target/example.wasm")
	instance.Init([]interface{}{commAddr})
	//assert.Equal(t, "success", res)

	hasPermission := instance.Invoke([]interface{}{"has_permission", commAddr, cAddr, "acl.createperm"})
	require.Equal(t,"true", hasPermission)

	lib.InvokerAddress = communityAddr
	res := instance.Invoke([]interface{}{"create_permission", clientAddress, appAddress2, "buy_book", managerAddress})
	require.Equal(t, "success", res)

	hasPermission = instance.Invoke([]interface{}{"has_permission", clientAddress, appAddress2, "buy_book"})
	require.Equal(t,"true", hasPermission)
	//
	hasPermission = instance.Invoke([]interface{}{"has_permission", clientAddress, appAddress, "buy_book"})
	require.Equal(t, "false", hasPermission)
}

func TestPermissionRevoke(t *testing.T)  {
	lib.ContractAddress = lib.NewAddress([]byte("addr1111111111111111"))
	lib.InvokerAddress = lib.NewAddress([]byte("addr2211111111111222"))
	//cAddr := lib.ContractAddress.ToString()
	commAddr := lib.InvokerAddress.ToString()

	instance := lib.LoadContract("../../acl/target/example.wasm")
	instance.Init([]interface{}{commAddr})

	res := instance.Invoke([]interface{}{"create_permission", clientAddress, appAddress, "buy_book", commAddr})
	require.Equal(t, "success", res)

	hasPermission := instance.Invoke([]interface{}{"has_permission", clientAddress, appAddress, "buy_book"})
	assert.Equal(t,"true", hasPermission)

	t.Log("error revoke case1")
	res = instance.Invoke([]interface{}{"revoke_permission", clientAddress, appAddress2, "buy_book"})
	require.Equal(t,"you don't have permission to revoke", res)

	t.Log("error revoke case2")
	res = instance.Invoke([]interface{}{"revoke_permission", appAddress2, appAddress, "buy_book"})
	fmt.Println(res)

	hasPermission = instance.Invoke([]interface{}{"has_permission", clientAddress, appAddress, "buy_book"})
	require.Equal(t,"true", hasPermission)

	t.Log("normal revoke case")
	res = instance.Invoke([]interface{}{"revoke_permission", clientAddress, appAddress, "buy_book"})
	fmt.Println(res)

	hasPermission = instance.Invoke([]interface{}{"has_permission", clientAddress, appAddress, "buy_book"})
	require.Equal(t,"false", hasPermission)
}


func TestPermissionGrant(t *testing.T)  {
	lib.ContractAddress = lib.NewAddress([]byte("addr1111111111111111"))
	lib.InvokerAddress = lib.NewAddress([]byte("addr2211111111111222"))
	//cAddr := lib.ContractAddress.ToString()
	commAddr := lib.InvokerAddress.ToString()

	instance := lib.LoadContract("../../acl/target/example.wasm")
	instance.Init([]interface{}{commAddr})

	res := instance.Invoke([]interface{}{"create_permission", clientAddress, appAddress, "buy_book", commAddr})
	require.Equal(t, "success", res)

	hasPermission := instance.Invoke([]interface{}{"has_permission", clientAddress, appAddress, "buy_book"})
	require.Equal(t,"true", hasPermission)

	res = instance.Invoke([]interface{}{"grant_permission", clientAddress2, appAddress, "buy_book"})

	hasPermission = instance.Invoke([]interface{}{"has_permission", clientAddress2, appAddress, "buy_book"})
	require.Equal(t,"true", hasPermission)
}

func TestPermissionSetManager(t *testing.T)  {
	lib.ContractAddress = lib.NewAddress([]byte("addr1111111111111111"))
	lib.InvokerAddress = lib.NewAddress([]byte("addr2211111111111222"))
	//cAddr := lib.ContractAddress.ToString()
	commAddr := lib.InvokerAddress.ToString()

	instance := lib.LoadContract("../../acl/target/example.wasm")
	instance.Init([]interface{}{commAddr})

	res := instance.Invoke([]interface{}{"create_permission", clientAddress, appAddress, "buy_book", commAddr})
	require.Equal(t, "success", res)
	hasPermission := instance.Invoke([]interface{}{"has_permission", clientAddress, appAddress, "buy_book"})
	require.Equal(t,"true", hasPermission)

	res = instance.Invoke([]interface{}{"set_permission_manager", appAddress, "buy_book", managerAddress})

	temp, _ := hex.DecodeString(managerAddress[2:])
	lib.InvokerAddress = lib.NewAddress(temp)
	res = instance.Invoke([]interface{}{"revoke_permission", clientAddress, appAddress, "buy_book"})
	fmt.Println(res)

	hasPermission = instance.Invoke([]interface{}{"has_permission", clientAddress, appAddress, "buy_book"})
	require.Equal(t,"false", hasPermission)
}

func TestDefaultPermissionList(t *testing.T) {
	lib.ContractAddress = lib.NewAddress([]byte("addr1111111111111111"))
	instance := lib.LoadContract("../../acl/target/example.wasm")
	res := instance.Invoke([]interface{}{"default_permissions"})
	require.Equal(t, "{\"acl.createperm\":\"0x6164647231313131313131313131313131313131\"}", res)
}

func TestInit(t *testing.T) {
	lib.ContractAddress = lib.NewAddress([]byte("addr1111111111111111"))
	lib.InvokerAddress = lib.NewAddress([]byte("addr2211111111111222"))
	//cAddr := lib.ContractAddress.ToString()
	commAddr := lib.InvokerAddress.ToString()

	instance := lib.LoadContract("../../acl/target/example.wasm")
	_ = instance.Invoke([]interface{}{"init", commAddr})

	res := instance.Invoke([]interface{}{"create_permission", clientAddress, appAddress, "buy_book", commAddr})
	require.Equal(t, "success", res)
	hasPermission := instance.Invoke([]interface{}{"has_permission", clientAddress, appAddress, "buy_book"})
	require.Equal(t,"true", hasPermission)

	res = instance.Invoke([]interface{}{"set_permission_manager", appAddress, "buy_book", managerAddress})

	temp, _ := hex.DecodeString(managerAddress[2:])
	lib.InvokerAddress = lib.NewAddress(temp)
	res = instance.Invoke([]interface{}{"revoke_permission", clientAddress, appAddress, "buy_book"})
	fmt.Println(res)

	hasPermission = instance.Invoke([]interface{}{"has_permission", clientAddress, appAddress, "buy_book"})
	require.Equal(t,"false", hasPermission)
}