package voting

import (
	"encoding/json"
	"fmt"
	"github.com/stretchr/testify/assert"
	"sdkTest/lib"
	"strconv"
	"testing"
)

const (
	acl = "0x9078382938473294324238204824323327492110"
	community = "0x1078382938473294324238204825323327492310"
	token  = "0x3422482938473294324238204824323327492323"
	voter1 = "0x329323804203482043170183208028301f830213"
	voter2 = "0x809323804203482043170183208028301f830693"
)

type Voter struct {
	Support     bool      `json:"support"`
	Stake       int64    `json:"stake"`
}

type Vote struct {
	Sponsor   			string    			`json:"sponsor"`
	Passed              bool                `json:"passed"`
	HasExecuted 		bool  				`json:"has_executed"`
	MinAcceptQuorumPct 	uint64  			`json:"min_accept_quorum_pct"`
	Nay      			uint64      		`json:"nay"`
	SnapShotBlock  		uint64  			`json:"snap_shot_block"`
	StartDate     		uint64   			`json:"start_date"`
	SupportRequiredPct  uint64  			`json:"support_required_pct"`
	VoteVotingPower   	uint64    			`json:"vote_voting_power"`
	Voters       		map[string]Voter  	`json:"voters"`
	Yea          		uint64          	`json:"yea"`
	Event               Event               `json:"event"`
}

type Event struct {
	EventType     string     `json:"event_type"`
	Contract      string     `json:"contract"`
	Method        string     `json:"method"`
	Params        []string   `json:"params"`
	Description   string     `json:"description"`
}
func NewEvent(ty, contract, method, des string, params []string) Event {
	return Event{
		EventType:   ty,
		Contract:    contract,
		Method:      method,
		Params:      params,
		Description: des,
	}
}

type PublicParams struct {
	MinAcceptQuorumPct     uint64      `json:"min_accept_quorum_pct"`
	SupportRequiredPct     uint64      `json:"support_required_pct"`
	VoteOpenTime           uint64      `json:"vote_open_time"`
}

type MiddleParams struct {
	MinAcceptQuorumPct     string      `json:"min_accept_quorum_pct"`
	SupportRequiredPct     string      `json:"support_required_pct"`
	VoteOpenTime           string      `json:"vote_open_time"`
}

func TestCastVoteAndChangeVote(t *testing.T) {
	//
	instance := lib.LoadContract("../../voting/target/voting.wasm")
	//acl address; community address; support_required_pct; min_accept_quorum_pct; vote_persistent_time;
	//10^8 -> 1%;
	//2 weeks = 1209600000;
	res := instance.Invoke([]interface{}{acl, community, "6000000000", "5000000000", "1209600000"})
	assert.Equal(t, res, "Success")

	//voter_address;
	var e = NewEvent("call_contract", "token_app", "mint", "mint token to account", []string{"0x329323804203482043170183208028301f830213", "100"})
	//var e2 = NewEvent("calim","", "", "add shard", nil)
	var event = fmt.Sprintf("{\"event_type\":\"%s\", \"contract\":\"%s\", \"method\":\"%s\", \"params\":\"%s\"},\"description\":\"%s\"",
		e.EventType, e.Contract, e.Method, e.Params, e.Description)
	res = instance.Invoke([]interface{}{"new_vote", event})
	assert.Equal(t, res, "1")

	//voter address; vote_id; supports?; executes_if_decided?;
	res = instance.Invoke([]interface{}{"cast_vote", uint64(1), true})
	assert.Equal(t, res, "Success")

	res  = instance.Invoke([]interface{}{"query_vote", uint64(1)})
	vote := toVote(res)

	PrintVote(vote)

	//change vote.
	res = instance.Invoke([]interface{}{"cast_vote", uint64(1), false})
	assert.Equal(t, res, "cast vote success")

	res  = instance.Invoke([]interface{}{"query_vote", uint64(1)})
	newVote := toVote(res)

	PrintVote(newVote)
}

func TestQueryVotes(t *testing.T) {
	//acl address; community address; support_required_pct; min_accept_quorum_pct; vote_persistent_time;
	//10^8 -> 1%;
	//2 weeks = 1209600000;
	instance := lib.LoadContract("../../voting/target/voting.wasm")
	res := instance.Invoke([]interface{}{acl, community, "6000000000", "5000000000", "1209600000"})
	assert.Equal(t, res, "init complete")

	var e = NewEvent("call_contract", "token_app", "mint", "mint token to account", []string{"0x329323804203482043170183208028301f830213", "100"})
	//var e2 = NewEvent("calim","", "", "add shard", nil)
	var event = fmt.Sprintf("{\"event_type\":\"%s\", \"contract\":\"%s\", \"method\":\"%s\", \"params\":\"%s\"},\"description\":\"%s\"",
		e.EventType, e.Contract, e.Method, e.Params, e.Description)
	res = instance.Invoke([]interface{}{"new_vote", event})
	assert.Equal(t, res, "1")
	//new vote_2
	//var e = NewEvent("call_contract", "token_app", "mint", "mint token to account", []string{"0x329323804203482043170183208028301f830213", "100"})
	var e2 = NewEvent("calim","", "", "add shard", nil)
	var event2 = fmt.Sprintf("{\"event_type\":\"%s\", \"contract\":\"%s\", \"method\":\"%s\", \"params\":\"%s\"},\"description\":\"%s\"",
		e2.EventType, e2.Contract, e2.Method, e2.Params, e2.Description)
	res = instance.Invoke([]interface{}{"new_vote", event2})
	assert.Equal(t, res, "vote_id = 2")

	//query all votes
	//no params
	res = instance.Invoke([]interface{}{"query_votes"})
	PrintAllVotes(res)
	//query vote.
	//vote_id;
	res  = instance.Invoke([]interface{}{"query_vote", uint64(1)})
	vote := toVote(res)

	PrintVote(vote)
}

func TestQueryPublicParams(t *testing.T) {
	//acl address; community address; support_required_pct; min_accept_quorum_pct; vote_persistent_time;
	//10^8 -> 1%;
	//2 weeks = 1209600000;
	instance := lib.LoadContract("../../voting/target/voting.wasm")
	res := instance.Invoke([]interface{}{acl, community, "6000000000", "5000000000", "1209600000"})
	assert.Equal(t, res, "init complete")

	//query params
	//no params.
	res = instance.Invoke([]interface{}{"query_public_params"})
	params := toPublicParams(res)
	PrintPublicParams(params)
}

func TestQueryDefaultPermissions(t *testing.T) {
	//acl address; community address; support_required_pct; min_accept_quorum_pct; vote_persistent_time;
	//10^8 -> 1%;
	//2 weeks = 1209600000;
	instance := lib.LoadContract("../../voting/target/voting.wasm")
	res := instance.Invoke([]interface{}{acl, community, "6000000000", "5000000000", "1209600000"})
	fmt.Printf(res)

	//query permissions.
	res = instance.Invoke([]interface{}{"default_permissions"})
	fmt.Printf(res)
}

func PrintVote(vote Vote) {
	fmt.Printf("Sponsor : %s\n", vote.Sponsor)
	fmt.Printf("HasExecuted : %v\n", vote.HasExecuted)
	fmt.Printf("MinAcceptQuorumPct : %v\n", vote.MinAcceptQuorumPct)
	fmt.Printf("SupportRequiredPct : %v\n", vote.SupportRequiredPct)
	fmt.Printf("VoteVotingPower : %v\n", vote.VoteVotingPower)
	fmt.Printf("StartDate : %v\n", vote.StartDate)
	fmt.Printf("SnapShotBlock : %v\n", vote.SnapShotBlock)
	fmt.Printf("Yea : %v\n", vote.Yea)
	fmt.Printf("Nay : %v\n", vote.Nay)
	fmt.Printf("Voters: %v\n", vote.Voters)
}

func PrintPublicParams(par PublicParams) {
	fmt.Printf("MinAcceptQuorumPct : %v\n", par.MinAcceptQuorumPct)
	fmt.Printf("SupportRequiredPct : %v\n", par.SupportRequiredPct)
	fmt.Printf("VoteOpenTime : %v\n", par.VoteOpenTime)
}

func PrintAllVotes(res string) {
	//parse return value.
	var resultMap map[string]string
	err := json.Unmarshal([]byte(res), &resultMap)
	if err != nil {
		panic(err)
	}
	for _, value := range resultMap {
		vote := toVote(value)
		PrintVote(vote)
	}
}

//transfer to Vote.
func toVote(res string) Vote {
	var v Vote
	err := json.Unmarshal([]byte(res), &v)
	if err != nil {
		panic(err)
	}
	return v
}

//transfer to params.
func toPublicParams(res string) PublicParams {
	//
	var midParams MiddleParams
	err := json.Unmarshal([]byte(res), &midParams)
	if err != nil {
		panic(err)
	}

	var params PublicParams
	params.SupportRequiredPct, err = strconv.ParseUint(midParams.SupportRequiredPct, 10, 64)
	if err != nil {
		panic(err)
	}
	params.MinAcceptQuorumPct, err = strconv.ParseUint(midParams.MinAcceptQuorumPct, 10, 64)
	if err != nil {
		panic(err)
	}
	params.VoteOpenTime, err = strconv.ParseUint(midParams.VoteOpenTime, 10, 64)
	if err != nil {
		panic(err)
	}

	return params
}


func TestUnmarshal(t *testing.T) {
	var a = "{\"sponsor\":\"0x3f43e75aaba2c2fd6e227c10c6e7dc125a93de3c\",\"passed\":true,\"executed\":true,\"start_date\":1600239251,\"end_date\":2890839251,\"snapshot_block\":2,\"support_required_pct\":6000000000,\"min_accept_quorum_pct\":5000000000,\"yea\":8000000,\"nay\":0,\"voting_power\":8000000,\"voters\":{\"0x3f43e75aaba2c2fd6e227c10c6e7dc125a93de3c\":{\"support\":true,\"stake\":8000000}},\"event\":{\"event_type\":\"claim\",\"contract\":\"acl_app\",\"method\":\"\",\"params\":[],\"description\":\"add shard\"}}"
	var b Vote
	err := json.Unmarshal([]byte(a), &b)
	if err != nil {
		panic(err)
	}
	fmt.Println(b)
}