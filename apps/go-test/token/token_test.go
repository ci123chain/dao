package token

import (
	"github.com/stretchr/testify/assert"
	"sdkTest/lib"
	"testing"
)


var (
    aclAddress = "0xD1a14962627fAc768Fe885Eeb9FF072706B54c19"

	clientAddress1 = "0x505A74675dc9C71eF3CB5DF309256952917E801e"
	clientAddress2 = "0x505A74675dc9C71eF3CB5DF309256952917E803e"

	clientAddress = "0x3F43E75Aaba2c2fD6E227C10C6E7DC125A93DE3c"
)

func TestInit(t *testing.T)  {
	instance := lib.LoadContract("../../token/target/example.wasm")

	res := instance.Invoke([]interface{}{"init", aclAddress, "token_name", "TOK", "2", clientAddress1, "100", clientAddress2, "50"})

	assert.Equal(t, res, "Success")
}

func TestMint(t *testing.T)  {
	instance := lib.LoadContract("../../token/target/example.wasm")

	instance.Invoke([]interface{}{"init", aclAddress, "token_name", "TOK", "2", clientAddress1, "100", clientAddress2, "50"})
	res := instance.Invoke([]interface{}{"mint", clientAddress1, "100"})

	assert.Equal(t, res, "Success")
}

func TestBurn(t *testing.T)  {
	instance := lib.LoadContract("../../token/target/example.wasm")

	instance.Invoke([]interface{}{"init", aclAddress, "token_name", "TOK", "2", clientAddress1, "100", clientAddress2, "50"})
	res := instance.Invoke([]interface{}{"burn", clientAddress2, "50"})

	assert.Equal(t, res, "Success")
}

func TestBalance(t *testing.T)  {
	instance := lib.LoadContract("../../token/target/example.wasm")

	instance.Invoke([]interface{}{"init", aclAddress, "token_name", "TOK", "2", clientAddress1, "100", clientAddress2, "50"})
	res := instance.Invoke([]interface{}{"balance", clientAddress2})

	assert.Equal(t, res, "50")
}

func TestTransfer(t *testing.T)  {
	instance := lib.LoadContract("../../token/target/example.wasm")

	instance.Invoke([]interface{}{"init", aclAddress, "token_name", "TOK", "2", clientAddress1, "100", clientAddress2, "50"})
	res := instance.Invoke([]interface{}{"transfer", clientAddress1, clientAddress2, "50"})

	assert.Equal(t, res, "Success")
}