extern crate c123chain_cdk as cdk;
extern crate serde_json;

use cdk::types::{Address, Response};
use cdk::{runtime, math};
use cdk::codec::{Sink, Source};
use cdk::util::clone_into_array;
use std::collections::HashMap;
use Clone;
use serde::{Deserialize, Serialize};
use serde_json::Error;
//use cdk::debug;
const ADDR_SIZE: usize = 20;
const EMPTY_ENTITY: [u8; ADDR_SIZE] = [0; 20];
const KEY_ACL_APP: &str = "acl_app";
const KEY_COMMUNITY_APP: &str = "community_app";
const KEY_HAS_TOKEN_ADDRESS: &str = "_has_set_token_address";
const KEY_IS_OFFICIAL: &str = "is_official";
const KEY_QUERY_APP: &str = "query_app";
const ACTION_SET_TOKEN_ADDRESS: &str = "voting.set_token_address";
const ACTION_UPDATE_COMMUNITY_VOTING_RULE: &str = "update_community_voting_rule";

//token contract address;
const KEY_TOKEN: &str = "token_app";
//required support percent;
const KEY_SUPPORT_REQUIRED_PCT: &str = "support_required_pct";
//min accept quorum percent;
const KEY_MIN_ACCEPT_QUORUM_PCT: &str = "min_accept_quorum_pct";
//vote open time;
const KEY_VOTE_TIME: &str = "vote_open_time";
//quantity of vote;
const KEY_VOTE_LENGTH: &str = "vote_length";
// pct_base = 10^18; 100%;
const PCT_BASE: u128 = 1e18 as u128;


#[no_mangle]
pub fn init() {
    let deps = runtime::make_dependencies();
    let input = deps.api.input();
    let input = Source::new(&input);
    let _acl = input.read_str().unwrap();
    let cm = input.read_str().unwrap();
    let _support_str = input.read_str().unwrap();
    let _support_required_pct = _support_str.parse::<u128>().unwrap();
    let _min_str = input.read_str().unwrap();
    let _min_accept_quorum_pct = _min_str.parse::<u128>().unwrap();
    let _vote_str = input.read_str().unwrap();
    let _vote_time = _vote_str.parse::<u64>().unwrap();
    let is_official_str = input.read_str().unwrap();
    let is_official: bool;
    match is_official_str {
        "true" => {
            is_official = true
        }
        "false" => {
            is_official = false
        }
        _ => {
            return_contract(Err("invalid param: is_official"));
            return;
        }
    }
    initialize(_acl, cm, _support_required_pct, _min_accept_quorum_pct, _vote_time, is_official);
    return_contract(Ok(Response { data: "success".as_bytes()}));
}

#[no_mangle]
pub fn invoke() {
    let deps = runtime::make_dependencies();
    let input = deps.api.input();
    let input = Source::new(&input);
    let method = input.read_str().unwrap();
    match method {
        "new_vote" => {
            let time = get_time();
            let mut vote = Vote::default();
            let mut voter = _invoker();
            let event = input.read_str().unwrap();
            let vote_id = vote.new_vote(&mut voter, true,  time, event);
            return_contract(Ok(Response { data: vote_id.to_string().as_bytes() }));
        }

        //cast vote
        "cast_vote" => {
            let vote_id: u64;
            let supports: bool;
            let vote_id_str = input.read_str().unwrap();
            vote_id = vote_id_str.parse::<u64>().unwrap();
            let supports_str = input.read_str().unwrap();
            match supports_str {
                "true" => {
                    supports = true
                }
                "false" => {
                    supports = false
                }
                _ => {
                    return_contract(Err("error params"));
                    return;
                }
            }
            let mut voter= _invoker();
            let ok = Vote::cast_vote(vote_id, supports, &mut voter);
            if ok {
                return_contract(Ok(Response { data: "success".as_bytes()}));
            }
        }

        "execute_event" => {
            let vote_id_str = input.read_str().unwrap();
            let (res, ok) = get_param(vote_id_str);
            if !ok {
                return_contract(Err("invalid vote_id, the vote not exist"));
                return
            }
            let mut vote: Vote = serde_json::from_slice(&res).unwrap();
            if vote.executed {
                return_contract(Err("the vote has been executed already"));
                return
            }
            if get_time() > vote.end_date {
                return_contract(Err("the vote has been out of date"));
                return
            }
            if vote.passed {
                match &(*vote.event.event_type) {
                    "claim" => {
                        //do nothing
                    }
                    "call_contract" => {
                        let invoker = _invoker();
                        let addr = invoker.to_string();
                        let ok = String::eq(&addr, &vote.sponsor);
                        if !ok {
                            return_contract(Err("address not match, the invoker must be similar to event.sponsor"));
                        }
                        let executed = vote.unsafe_execute_vote();
                        if executed {
                            vote.executed = true;
                            let res = serde_json::to_vec(&vote).unwrap();
                            set_param(vote_id_str, &res);
                        }else {
                            return
                        }
                    }
                    "community" => {
                        let executed = vote.unsafe_execute_vote();
                        if executed {
                            vote.executed = true;
                            let res = serde_json::to_vec(&vote).unwrap();
                            set_param(vote_id_str, &res);
                        }else {
                            return
                        }
                    }
                    _ => {
                        //
                    }
                }
                return_contract(Ok(Response{ data: "success".as_bytes() }));
            }else {
                return_contract(Err("invalid vote_id, the vote not passed yet"));
            }
        }

        "default_permissions" => {
            let permissions = default_permissions();
            let res = serde_json::to_vec(&permissions).unwrap();
            return_contract(Ok(Response {data : &res}));
        }

        "query_votes" => {
            query_votes();
        }
        "query_vote" => {
            let vote_id = input.read_str().unwrap();
            let id = vote_id.parse::<u64>().unwrap();
            query_vote(id);
        }

        "query_public_params" => {
            query_public_params();
        }

         _ => {
            // 返回Error
            return_contract(Err("invoke method not found"));
        }
    }
}

#[no_mangle]
pub fn query() {
    let deps = runtime::make_dependencies();
    let input = deps.api.input();
    let input = Source::new(&input);
    let method = input.read_str().unwrap();
    match method {
        "query_votes" => {
            query_votes();
        }
        "query_vote" => {
            let vote_id = input.read_str().unwrap();
            let id = vote_id.parse::<u64>().unwrap();
            query_vote(id);
        }

        "query_public_params" => {
            query_public_params();
        }
        _ => {
            // 返回Error
            return_contract(Err("invoke method not found"));
        }
    }
}


fn initialize(_acl: &str, cm: &str, _support_required_pct: u128, _min_accept_quorum_pct:u128, _vote_time: u64, is_official: bool) {

    let vote_length: u64 = 0;
    //set params
    set_param(KEY_ACL_APP, _acl.as_bytes());
    set_param(KEY_COMMUNITY_APP, cm.as_bytes());
    set_param(KEY_SUPPORT_REQUIRED_PCT, &_support_required_pct.to_le_bytes());
    set_param(KEY_MIN_ACCEPT_QUORUM_PCT, &_min_accept_quorum_pct.to_le_bytes());
    set_param(KEY_VOTE_TIME, &_vote_time.to_le_bytes());
    set_param(KEY_VOTE_LENGTH, &(vote_length.to_le_bytes()));

    let mut res_sink = Sink::new(0);
    res_sink.write_bool(false);
    set_param(KEY_HAS_TOKEN_ADDRESS, &res_sink.into());

    let mut official_sink = Sink::new(0);
    official_sink.write_bool(is_official);
    set_param(KEY_IS_OFFICIAL, &official_sink.into())

}

fn query_votes() {
    let (length, _) = get_param(KEY_VOTE_LENGTH);
    let vote_length = u64::from_le_bytes(clone_into_array(&length));
    let mut res_map: HashMap<u64, Vote> = HashMap::new();
    let mut i: u64 = 1;
    while i <= vote_length {
        let id = i.to_string();
        let (res, _) = get_param(&id);
        let vote = serde_json::from_slice(&res).unwrap();
        res_map.insert(i, vote);
        i = i + 1;
    }
    let all_res = serde_json::to_vec(&res_map).unwrap();
    return_contract(Ok(Response { data: &all_res }));
}

fn _invoker() ->Address {
    let deps = runtime::make_dependencies();
    return deps.api.get_invoker();
}

fn _pre_caller() -> Address {
    let deps = runtime::make_dependencies();
    return deps.api.get_pre_caller()
}

fn query_vote(vote_id: u64) {
    let id = vote_id.to_string();
    let (res, ok) = get_param(&id);
    if !ok {
        return_contract(Err("invalid vote_id, the vote not exist"));
    }else {
        return_contract(Ok(Response{ data: &res}));
    }
}

fn query_public_params() {
    let (support_required_pct, _) = get_param(KEY_SUPPORT_REQUIRED_PCT);
    let (min_accept_quorum_pct, _) = get_param(KEY_MIN_ACCEPT_QUORUM_PCT);
    let (vote_open_time, _) = get_param(KEY_VOTE_TIME);
    let _support_required_pct = clone_into_array(&support_required_pct);
    let _min_accept_quorum_pct = clone_into_array(&min_accept_quorum_pct);
    let _vote_open_time = clone_into_array(&vote_open_time);
    let mut map:HashMap<&str, String> = HashMap::new();
    map.insert(KEY_VOTE_TIME, (u64::from_le_bytes(_vote_open_time)).to_string());
    map.insert(KEY_MIN_ACCEPT_QUORUM_PCT, (u128::from_le_bytes(_min_accept_quorum_pct)).to_string());
    map.insert(KEY_SUPPORT_REQUIRED_PCT, (u128::from_le_bytes(_support_required_pct)).to_string());

    let res = serde_json::to_vec(&map).unwrap();
    return_contract(Ok(Response{ data: &res}));
}

#[derive(Clone, Default, Debug, PartialEq)]
struct MiniMeToken{
    address:Address
}

impl MiniMeToken {
    fn new_from_address(add: Address) -> MiniMeToken {
        return MiniMeToken{
            address: add
        }
    }
    fn balance_of_at(&self, voter: &mut Address) -> u128 {
        let deps = runtime::make_dependencies();
        let token_address = self.address;
        let mut sink = Sink::new(0);
        let a = voter.to_string();
        sink.write_str("balance");
        sink.write_str(&a);
        let input = sink.into();

        match deps.api.call_contract(&token_address, &input) {
                Some(res) => {
                    let result = String::from_utf8(res).unwrap();
                    let balance = (*result).parse::<u128>().unwrap();
                    return balance
                },
                None => {
                    let ret: u128 = 0;
                    return ret
                },
        }
    }

    fn total_supply_at(&self) -> u128 {
        let deps = runtime::make_dependencies();
        let token_address = self.address;
        let mut sink = Sink::new(0);
        sink.write_str("totalSupply");
        let input = sink.into();
        match deps.api.call_contract(&token_address, &input) {
                Some(res) => {
                    let result = String::from_utf8(res).unwrap();
                    let total = (*result).parse::<u128>().unwrap();
                    return total
                },
                None => {
                    let ret: u128 = 0;
                    return ret;
                },
        };
    }
}

#[derive(Serialize, Deserialize, Default, Copy, Clone)]
struct Voter {
    support: bool,
    stake: u128,
}


#[derive(Serialize, Deserialize, Default)]
struct Event {
    event_type: String,
    contract: String,
    method: String,
    params: Vec<String>,
    description: String,
}


#[derive(Serialize, Deserialize, Default)]
//vote
struct Vote {
    sponsor: String,  //Sponsor
    passed: bool,
    executed:bool,    //has been executed?
    start_date:u64,   //start time
    end_date: u64,    //end time
    snapshot_block:u64,   //start block height;
    support_required_pct:u128,  //required supported percent;
    min_accept_quorum_pct:u128, //min accept quorum percent;
    yea:u128,      //supports
    nay:u128,      //opposite
    voting_power:u128,   //voting_power
    voters: HashMap<Address, Voter>,   //voter[voter_address, vote_state]
    event: Event,
}
//vote function
impl Vote {

    fn set_params(&mut self) {
        let (support_required_pct_u8, _) = get_param(KEY_SUPPORT_REQUIRED_PCT);
        let (min_accept_quorum_pct_u8, _) = get_param(KEY_MIN_ACCEPT_QUORUM_PCT);
        let support_required_pct = u128::from_le_bytes(clone_into_array(&support_required_pct_u8));
        let min_accept_quorum_pct = u128::from_le_bytes(clone_into_array(&min_accept_quorum_pct_u8));
        self.support_required_pct = support_required_pct;
        self.min_accept_quorum_pct = min_accept_quorum_pct;
    }
   
    //return voteId
    pub fn new_vote(&mut self, voter:&mut Address, cast_vote: bool, time: u64, event: &str) -> u64 {
        let (vote_length, _) = get_param(KEY_VOTE_LENGTH);
        let length = u64::from_le_bytes(clone_into_array(&vote_length));
        let vote_id = length + 1;
        //get chain block height;
        let block_number = get_block_height();
        let power;
        if is_official() {
            power = get_total_power();
        }else {
            let (voting_power, ok) = _get_total_supply();
            if !ok {
                return 0;
            }
            power = voting_power
        }
        
        let sponsor = voter.to_string();
        self.sponsor = sponsor;
        self.voting_power = power;
        self.set_params();   //set default vote params;
        self.start_date = time;
        let (open_time, _) = get_param(KEY_VOTE_TIME);
        let vote_time = u64::from_le_bytes(clone_into_array(&open_time));
        self.end_date = math::safe_add(time, vote_time);
        self.snapshot_block = block_number;
        let events: Result<Event, Error> = serde_json::from_str(event);
        match events {
            Err(_) => {
                return_contract(Err("error, invalid event"));
                return 0;
            }
            _ => {
                let e = events.unwrap();
                match &*e.event_type {
                    "call_contract" => {
                        let (com_addr, ok) = get_param(KEY_COMMUNITY_APP);
                        if !ok {
                            return_contract(Err("community contract not set yet"));
                            return 0;
                        }
                        let addr = String::from_utf8(com_addr).unwrap();
                        let community = Address::from(addr.as_str());
                        let mut contract_sink = Sink::new(0);
                        contract_sink.write_str("query_app");
                        contract_sink.write_str(&e.contract);
                        let contract_input = contract_sink.into();
                        let (_, ok) = _get_contract_address(community, contract_input);
                        if !ok {
                            return_contract(Err("error, invalid contract in event"));
                            return 0;
                        }
                    }
                    "claim" => {
                        //do nothing.
                    }

                    "community" => {
                        if is_official() {
                            return_contract(Err("error, official contract can\'t update default voting params now"));
                            return 0;
                        }
                        if e.method != ACTION_UPDATE_COMMUNITY_VOTING_RULE {
                            return_contract(Err("invalid param: event.method"));
                            return 0;
                        }
                        if e.params.len() != 3 {
                            return_contract(Err("invalid params length"));
                            return 0;
                        }
                        let support_str = e.params[0].clone();
                        if support_str != "-1" {
                            let res = support_str.parse::<u128>();
                            match res {
                                Err(_) =>  {
                                    return_contract(Err("error params: support_pct"));
                                    return 0;
                                }
                                _ => {
                                    //do nothing
                                }
                            }
                        }
                        let min_required_str = e.params[1].clone();
                        if min_required_str != "-1" {
                            let res = min_required_str.parse::<u128>();//.unwrap();
                            match res {
                                Err(_) => {
                                    return_contract(Err("error param: min_required_pct"));
                                    return 0;
                                }
                                _ => {
                                    //do nothing
                                }
                            }
                        }
                        let open_time_str = e.params[2].clone();
                        if open_time_str != "-1" {
                            let res = open_time_str.parse::<u64>();//.unwrap();
                            match res {
                                Err(_) => {
                                    return_contract(Err("error param: vote_open_time"));
                                    return 0;
                                }
                                _ => {
                                    //do nothing
                                }
                            }
                        }

                    }
                    _ => {
                        //
                    }
                }
                self.event = e;
            }
        }
        if cast_vote && self.can_vote(voter) {
            //cast voe
            let ok = self.vote(voter, true);
            if !ok {
                return 0;
            }
        }
        //save vote
        //vore_id -> vote
        let rep = serde_json::to_vec(&self).unwrap();
        let id = vote_id.to_string();
        set_param(&id, &rep);
        //save vote length
        set_param(KEY_VOTE_LENGTH, &(vote_id.to_le_bytes()));
        return vote_id
    }

    pub fn cast_vote(vote_id: u64, supports: bool, voter: &mut Address) -> bool {
        let id = vote_id.to_string();
        let (res, ok) = get_param(&id);
        if !ok {
            return_contract(Err("the vote not existed"));
            return false
        }
        //unMarshal.
        let mut vote: Vote = serde_json::from_slice(&res).unwrap();
        let power;
        if is_official() {
            power = get_total_power();
        }else {
            let (voting_power, ok) = _get_total_supply();
            if !ok {
                return false;
            }
            power = voting_power
        }
        vote.voting_power = power;
        if !vote.can_vote(voter) {
            return false;
        }
        let ok = vote.vote(voter, supports);
        if !ok {
            return false
        }
        //save vote
        //vore_id -> vote
        let res = serde_json::to_vec(&vote).unwrap();
        set_param(&id, &res);
        return true
    }

    fn can_vote(&self, voter: &mut Address) -> bool {
        if is_official() {
            let validators: &[&Address] = &[voter];
            let resp = get_validator_power(validators);
            if resp[0] == 0{
                return_contract(Err("you have no power to cast vote"));
                return false
            }
        }else {
            let (community_address_vec, _) = get_param(KEY_COMMUNITY_APP);
            let a = String::from_utf8(community_address_vec).unwrap();
            let community_address = Address::from(a.as_str());
            let (token_address, ok) = query_contract_address(community_address, KEY_TOKEN);
            let token = MiniMeToken::new_from_address(token_address);
            if !ok {
                return_contract(Err("not found token app, you must install token app first"));
                return false
            }
            if token.balance_of_at(voter) == 0 {
                return_contract(Err("you have no power to cast vote"));
                return false
            }
        }
        if !self.is_vote_open(){
            return_contract(Err("the vote is not open"));
            return false
        }
        return true
    }

    fn can_pass(&mut self) -> bool {
        if self.executed {
            //executed already.
            return false
        }
        if self.passed {
            //have passed already.
            return false
        }
        //// Vote ended?
        if !self.is_vote_open(){
            //not open
            return false
        }
        if is_official() {
            self.voting_power = get_total_power();
            let current_voters = self.voters.clone();
            let mut validators: Vec<&Address> = Vec::new();
            for (k, _) in current_voters.iter() {
                validators.push(k);
            }
            //let b = &validators;
            let resp = get_validator_power(&validators);
            for (i, _v) in resp.iter().enumerate() {
                let addr = validators.get(i).unwrap();
                let voter = self.voters.get(*addr).unwrap();
                let mut stake = voter.clone();
                if stake.stake != *_v {
                    match stake.support {
                        true => {
                            self.yea = math::safe_sub(self.yea, stake.stake);
                            self.yea = math::safe_add(self.yea, *_v)
                        }
                        false => {
                            self.nay = math::safe_sub(self.nay, stake.stake);
                            self.nay = math::safe_add(self.nay, *_v)
                        }
                    }
                }
                stake.stake = *_v;
                self.voters.insert(**addr, stake);
            }
        }
        let total_votes = math::safe_add(self.yea, self.nay);  
        // Has enough support?
        if !(self.is_value_pct(self.yea, total_votes, self.support_required_pct)) {
            //
            return false
        }
        // Has min quorum?
        if !(self.is_value_pct(self.yea, self.voting_power, self.min_accept_quorum_pct)) {
            return false
        }
        return true
    }

    //return true if the given vote is open, false otherwise
    fn is_vote_open(&self) -> bool {
        let time_stamp = get_time();
        return time_stamp < self.end_date && !self.executed && !self.passed;
    }

    //Calculates whether `_value` is more than a percentage `_pct` of `_total`
    fn is_value_pct(&self, value: u128, total: u128, pct: u128) -> bool {
        if total == 0 || value == 0 {
            return false
        }
        let compucted_pct = math::safe_mul(value, PCT_BASE) / total;
        return compucted_pct > pct
    }

    fn vote(&mut self, voter: &mut Address, support: bool) -> bool {
        let address = voter.clone();
        let mut voter_state = Voter::default();
        let v = self.voters.get(&address);
        match v {
            None => {
                //do nothing
            }
            Some(value) => {
                voter_state.stake = value.stake;
                voter_state.support = value.support;
                let per_stake = voter_state.stake;
                let per_support = voter_state.support;
                match per_support {
                    true => {
                        self.yea = math::safe_sub(self.yea, per_stake);
                    }
                    false => {
                        self.nay = math::safe_sub(self.nay, per_stake);
                    }
                }
            }
        }
        let voter_current_stake: u128;
        if is_official() {
            let validators: &[&Address] = &[&voter];
            let resp = get_validator_power(validators);
            voter_current_stake = resp[0]
        }else {
            let (community_address_vec, _) = get_param(KEY_COMMUNITY_APP);
            let a = String::from_utf8(community_address_vec).unwrap();
            let community_address = Address::from(a.as_str());
            let (token_address, ok) = query_contract_address(community_address, KEY_TOKEN);
            let token = MiniMeToken::new_from_address(token_address);
            if !ok {
                return_contract(Err("not found token app, you must install token app first"));
                return false
            }
            voter_current_stake = token.balance_of_at(voter);
        }
        if support {
            self.yea = math::safe_add(self.yea, voter_current_stake);
            voter_state.support = true;
        }else {
            self.yea = math::safe_add(self.yea, voter_current_stake);
            voter_state.support = false;
        }
        voter_state.stake = voter_current_stake;
        self.voters.insert(address, voter_state);
        if self.can_pass() {
            self.passed = true;
            match &*self.event.event_type {
                "claim" => {
                    self.executed = true
                }
                _ => {
                    //
                }
            }
        }
        return true
    }

    fn unsafe_execute_vote(&mut self) -> bool {
        //execute vote
        //let event: Event = serde_json::from_str(&(*self.events)).unwrap();
        let action = self.event.event_type.clone();
        match &*action {
            "call_contract" => {
                let (com_addr, _) = get_param(KEY_COMMUNITY_APP);
                let addr = String::from_utf8(com_addr).unwrap();
                let community = Address::from(addr.as_str());
                let mut contract_sink = Sink::new(0);
                contract_sink.write_str("query_app");
                contract_sink.write_str(&self.event.contract);
                let contract_input = contract_sink.into();
                let (contract_addr, _) = _get_contract_address(community, contract_input);
                let mut sink = Sink::new(0);
                sink.write_str(&(*self.event.method));
                for i in &self.event.params {
                    sink.write_str(i);
                }
                let input = sink.into();
                let ok = _call_contract(contract_addr, input);
                if !ok {
                    return false
                }
            }
            "claim" => {
                //TODO
            }
            "community" => {
                //
                let support_str = self.event.params[0].clone();
                if support_str != "-1" {
                    let support = support_str.parse::<u128>().unwrap();
                    change_support_required_pct(support)
                }
                let min_required_str = self.event.params[1].clone();
                if min_required_str != "-1" {
                    let min_required = min_required_str.parse::<u128>().unwrap();
                    change_min_accept_quorum(min_required)
                }
                let open_time_str = self.event.params[2].clone();
                if open_time_str != "-1" {
                    let open_time = open_time_str.parse::<u64>().unwrap();
                    change_vote_open_time(open_time)
                }
            }
            _ => {
                //TODO
            }
        }
        self.executed = true;
        return true
    }
}


//chang public param
fn change_vote_open_time(new_vote_time: u64) {
    set_param(KEY_VOTE_TIME, &(new_vote_time.to_le_bytes()));
}
//chang public param
fn change_min_accept_quorum(min_accept_quorum_pct: u128) {
    set_param(KEY_MIN_ACCEPT_QUORUM_PCT, &(min_accept_quorum_pct.to_le_bytes()));
}
//chang public param
fn change_support_required_pct(supported_required_pct: u128) {
    set_param(KEY_SUPPORT_REQUIRED_PCT, &supported_required_pct.to_le_bytes());
}


fn default_permissions() -> HashMap<String, String> {
    let (addr, _) = get_param(KEY_COMMUNITY_APP);
    let address = String::from_utf8(addr).unwrap();
    let mut map: HashMap<String, String> = HashMap::new();
    map.insert(String::from(ACTION_SET_TOKEN_ADDRESS),address);
    return map
} 

fn query_contract_address(community_address: Address, app_name: &str) -> (Address, bool) {
    let mut input = Sink::new(0);
    input.write_str(KEY_QUERY_APP);
    input.write_str(app_name);
    let ret_input = input.into();
    let deps = runtime::make_dependencies();
    let address = Address::default();
    match deps.api.call_contract(&community_address, &*ret_input) {
        None => {
            return_contract(Err("call contract error, return none"));
            return (address, false)
        }
        Some(res) => {
            let a = String::from_utf8(res).unwrap();
            let app_address = Address::from(a.as_str());
            return (app_address, true)
        }
    }
}

fn _contract_address() -> Address {
    let deps = runtime::make_dependencies();
    let app = deps.api.self_address();
    return app
}

fn set_param(key: &str, value: &[u8]) {
    runtime::make_dependencies()
        .storage
        .set(key.as_bytes(), value)
}

fn get_param(key: &str) -> (Vec<u8>, bool) {
    let val = runtime::make_dependencies()
        .storage
        .get(key.as_bytes());
    match val {
        Some(value) => {
            return (value, true)
        }
        None => {
            return (Vec::new(), false)
        }
    }
}

fn get_block_height() -> u64 {
    let block = runtime::make_dependencies().api.get_block_header();
    return block.height;
}

fn get_time() -> u64 {
    let block = runtime::make_dependencies().api.get_block_header();
    return block.timestamp;
}

fn _call_contract(app_address: Address, input: Vec<u8>) -> bool {
    let ret_input = &*input;
    let deps = runtime::make_dependencies();
    match deps.api.call_contract(&app_address, ret_input) {
        Some(res) => {
            let ret = String::from_utf8(res).unwrap();
            match &*ret {
                "success" => {
                    return true
                }
                _ => {
                    return_contract(Err(&ret));
                    return false
                }
            }
        }
        None => {
            return_contract(Err("call contract error, return none"));
            return false
        }
    }
}

fn _get_contract_address(app_address: Address, input: Vec<u8>) -> (Address, bool) {
    let ret_input = &*input;
    let deps = runtime::make_dependencies();
    match deps.api.call_contract(&app_address, ret_input) {
        Some(res) => {
            let app_str = String::from_utf8(res).unwrap();
            let app = Address::from(app_str.as_str());
            return (app, true)
        }
        None => {
            return (Address::new(&EMPTY_ENTITY).unwrap(),false)
        },
    }
}

fn _get_total_supply() -> (u128, bool) {
    let voting_power: u128;
    if is_official() {
        voting_power = get_total_power();
    }else {
        let (community_address_vec, _) = get_param(KEY_COMMUNITY_APP);
        let a = String::from_utf8(community_address_vec).unwrap();
        let community_address = Address::from(a.as_str());
        let (token_address, ok) = query_contract_address(community_address, KEY_TOKEN);
        if !ok {
            return_contract(Err("not found token app, you must install token app first"));
            return (0, false)
        }
        let token = MiniMeToken::new_from_address(token_address);
        voting_power = token.total_supply_at();
    }
    if voting_power <= 0 {
        return_contract(Err("error, no voting power"));
        return (0, false);
    }
    return (voting_power, true)
}

fn return_contract<'a>(result: Result<Response, &'a str>) {
    runtime::make_dependencies().api.ret(result)
}

fn is_official() -> bool {
    let (res, _) = get_param(KEY_IS_OFFICIAL);
    let source = Source::new(&res);
    let resp = source.read_bool().unwrap();
    return resp
}
//get chain validator power.
fn get_validator_power(validators: &[&Address]) -> Vec<u128> {
    let deps = runtime::make_dependencies();
    let resp =deps.api.get_validator_power(validators);
    resp
}

//get chain total power.
fn get_total_power() -> u128 {
    let deps = runtime::make_dependencies();
    let resp =deps.api.total_power();
    resp
}

