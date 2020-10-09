extern crate c123chain_cdk as cdk;
extern crate serde_json;

use std::collections::HashMap;
use cdk::math;
use cdk::runtime;
//use cdk::runtime::ItemValue::Str as IString;
use cdk::runtime::Dependencies;
use cdk::types::{Address, Response};
use cdk::codec::{Source, Sink};
use serde::{Deserialize, Serialize};
use serde_json::Error;
const APP: &str = "token.";
const ADDR_SIZE: usize = 20;
const ANY_ENTITY: [u8; ADDR_SIZE] = [1; 20];
//const ANY_ENTITY: &str = "0x0000000000000000000000000000000000000001";

const VOTE_ADDR: &str = "vote";
const ACL_ADDR: &str = "acl_app";
const COMMUNITY_ADDR: &str = "community_app";

const TOKEN_NAME: &str = "token_name";
const TOKEN_SYM: &str = "token_symbol";
const TOTAL: &str = "total";
const BALANCE: &str = "balance:";

const KEY_QUERY_APP: &str = "query_app";

const HOLDER_LIST: &str = "holder_list";
//const EXIST: &str = "exist";
//const HAS_PERMISSION: &str = "has_permission";
const SUCCESS: &str = "success";

#[derive(Serialize, Deserialize, Default)]
struct GenesisAccount {
    address: String,
    amount: String,
}

#[no_mangle]
pub fn init() {
    let deps = runtime::make_dependencies();
    let input = deps.api.input();
    let input = Source::new(&input);
    set_system_contract(&deps, &input);
    _init(&deps, &input);
    return
}

#[no_mangle]
pub fn invoke() {
    let deps = runtime::make_dependencies();
    let input = deps.api.input();
    let input = Source::new(&input);
    let method = input.read_str().unwrap();
    match method {
        "default_permissions" => {
            default_permissions(&deps, method, &input);
            return
        }

        "mint" => {
            has_permission(&deps, &(APP.to_owned() + method));
            mint(&deps, method, &input);
            return
        }

        "burn" => {
            has_permission(&deps, &(APP.to_owned() + method));
            burn(&deps, method, &input);
            return
        }

        "balance" => {
            balance(&deps, method, &input);
        }

        "transfer" => {
            has_permission(&deps, &(APP.to_owned() + method));
            transfer(&deps, method, &input);
        }

        "totalSupply" => {
            total_supply(&deps, method, &input);
        }

        "holderList" => {
            holder_list(&deps, method, &input);
        }

        //test
        "canMigrate" => {
            let res = "true".as_bytes();
            return_contract(Ok(Response {
                data: res,
            }));
        }

        "call_demo_readDB" => {
            let addr: Address = input.read_str().unwrap().into();
            let method = input.read_str().unwrap();
            let key = input.read_str().unwrap();
            let mut sink = Sink::new(0);
            sink.write_str(method);
            sink.write_str(key);
            let ret_input = sink.into();
            match deps.api.call_contract(&addr, &ret_input) {
                Some(res) => return_contract(Ok(Response { data: &res })),
                None => return_contract(Err("call contract error")),
            }
        }

        _ => {
            // 返回Error
            return_contract(Err("invoke method not found"));
        }
    }
}

/*
fn event(method: &str, msg: &str) {
    let mut event = runtime::Event::new(method);
    event.add("msg", IString(msg));
    runtime::make_dependencies().api.notify(&event);
}
*/

fn read_db(key: &str) -> String {
    let val = runtime::make_dependencies()
        .storage
        .get(key.as_bytes());
    match val {
        None => {
            String::new()
        }
        Some(val_s) => {
            String::from_utf8(val_s).unwrap()
        }
    }
}

fn write_db(key: &str, value: &str) {
    runtime::make_dependencies()
        .storage
        .set(key.as_bytes(), value.as_bytes())
}
/*
fn delete_db(key: &str) {
    runtime::make_dependencies().storage.delete(key.as_bytes())
}
*/

fn return_contract<'a>(result: Result<Response, &'a str>) {
    runtime::make_dependencies().api.ret(result)
}

fn set_system_contract(deps: &Dependencies, input: &Source) {
    if read_db(ACL_ADDR).len() != 0 || read_db(COMMUNITY_ADDR).len() != 0 {
        return_contract(Err("It's already initialized"));
    }
    let acl_addr:Address = input.read_str().unwrap().into();
    let community_addr:Address = input.read_str().unwrap().into();
    write_db(ACL_ADDR, &acl_addr.to_string());
    write_db(COMMUNITY_ADDR, &community_addr.to_string());
}

fn default_permissions(deps: &Dependencies, method: &str, input: &Source) {
    let community_addr_str = read_db(COMMUNITY_ADDR);
    let community_addr_str = community_addr_str.as_str();
    let community_addr = Address::from(community_addr_str);
    let mut perm_list:HashMap<String, String> = HashMap::new();
    let (vote_addr, ok) = query_contract_address(community_addr, VOTE_ADDR);
    if !ok {
        return_contract(Err("vote app not install yet"));
        return
    }
    perm_list.insert(String::from("token.mint"), vote_addr.to_string());
    perm_list.insert(String::from("token.burn"), vote_addr.to_string());
    perm_list.insert(String::from("token.transfer"), Address::new(&ANY_ENTITY).unwrap().to_string());
    //perm_list.insert(String::from("token.balance"), String::from(ANY_ENTITY));
    //perm_list.insert(String::from("token.totalSupply"), String::from(ANY_ENTITY));
    let perm_list_v = serde_json::to_vec(&perm_list).unwrap();
    let perm_list_u8 = perm_list_v.as_slice();
    return_contract(Ok(Response {
        data: perm_list_u8,
    }));
}

//init args["tokenName","tokenSymbol","number",("address","amount")...]
//实例化合约 设置tokenName, tokenSymbol，初始发行[]amount 给 []address
fn _init(deps: &Dependencies, input: &Source) {
    let token_name = input.read_str().unwrap();
    let token_symbol = input.read_str().unwrap();

    // let mut addr:Vec<&str> = Vec::new();
    // let mut amount:Vec<u128> = Vec::new();
    let mut total:u128 = 0;
    let genesis_account: Result<Vec<GenesisAccount>, Error> = serde_json::from_str(input.read_str().unwrap());
    match genesis_account {
        Err(_) => {
            return_contract(Err("error, invalid account"));
        }
        _ => {
            let accounts = genesis_account.unwrap();
            for account in accounts.iter() {
                let address= &account.address;
                let amount_str = &account.amount;
                let amount = parse_u128(amount_str);
                let addr = Address::from(address.as_str()).to_string();

                let key = BALANCE.to_owned() + &addr;
                write_db(key.as_str(), amount_str);
                add_to_holder_list(&addr);
                total = math::safe_add(total, amount);
            }

        }
    }

    write_db(TOKEN_NAME, token_name);
    write_db(TOKEN_SYM, token_symbol);
    write_db(TOTAL, total.to_string().as_str());

    let res = SUCCESS.as_bytes();
    return_contract(Ok(Response {
        data: res,
    }));
}

//mint args["recAddr","amount"]
//增发代币 增发amount 给 recAddr
fn mint(deps: &Dependencies, method: &str, input: &Source) {
    let rec_str = addr_to_lower(input.read_str().unwrap());
    let rec_addr = rec_str.as_str();
    let amount = parse_u128(input.read_str().unwrap());

    let mut balance:u128;
    let key = BALANCE.to_owned() + rec_addr;
    let balance_str = read_db(key.as_str());
    // event(method, key.as_str());

    if balance_str.len() == 0 {
        balance = 0;
    }else {
        balance = parse_u128(balance_str.to_string().as_str())
    }

    balance = math::safe_add(balance, amount);

    write_db((BALANCE.to_owned() + rec_addr).as_str(), balance.to_string().as_str());

    let mut total:u128;
    let total_str = read_db(TOTAL.to_owned().as_str());
    if total_str.len() == 0 {
        total = 0;
    }else {
        total = parse_u128(total_str.as_str())
    }
    total = math::safe_add(total, amount);
    write_db(TOTAL.to_owned().as_str(), total.to_string().as_str());

    add_to_holder_list(rec_addr);

    let res = SUCCESS.as_bytes();
    return_contract(Ok(Response {
        data: res,
    }));
}

//burn args["recAddr","amount"]
//减少代币 减少amount 在 recAddr
fn burn(deps: &Dependencies, method: &str, input: &Source) {
    let rec_str = addr_to_lower(input.read_str().unwrap());
    let rec_addr = rec_str.as_str();
    let amount = parse_u128(input.read_str().unwrap());

    let mut balance:u128 = read_balance(rec_addr);

    balance = math::safe_sub(balance, amount);

    write_db((BALANCE.to_owned() + rec_addr).as_str(), balance.to_string().as_str());

    let mut total:u128;
    let total_str = read_db(TOTAL.to_owned().as_str());
    if total_str.len() == 0 {
        total = 0;
    }else {
        total = parse_u128(total_str.as_str())
    }

    total = math::safe_sub(total, amount);

    write_db(TOTAL, total.to_string().as_str());

    let res = SUCCESS.as_bytes();
    return_contract(Ok(Response {
        data: res,
    }));
}

//balance params["addr"]
//查询addr余额
fn balance(deps: &Dependencies, method: &str, input: &Source) {
    let addr = input.read_str().unwrap();
    let balance:u128 = read_balance(addr);
    return_contract(Ok(Response {
        data: balance.to_string().as_str().as_bytes(),
    }))
}

//transfer params["from","to","amount"]
//由from向to发起，额度为amount的转账
fn transfer(deps: &Dependencies, method: &str, input: &Source) {
    let from_str = addr_to_lower(input.read_str().unwrap());
    let from = from_str.as_str();
    let to_str = addr_to_lower(input.read_str().unwrap());
    let to = to_str.as_str();
    let amount = parse_u128(input.read_str().unwrap());

    let mut from_balance = read_balance(from);
    if from_balance < amount {
        return_contract(Err("Insufficient from balance"));
        return
    } else {
        //balance enough
        from_balance = math::safe_sub(from_balance, amount);
        write_db((BALANCE.to_owned() + from).as_str(), from_balance.to_string().as_str());
        let mut to_balance = read_balance(to);

        to_balance = math::safe_add(to_balance, amount);
        write_db((BALANCE.to_owned() + from).as_str(), to_balance.to_string().as_str());
    }
    add_to_holder_list(to);
    let res = SUCCESS.as_bytes();
    return_contract(Ok(Response {
        data: res,
    }));
}

//获取token总供应量
fn total_supply(deps: &Dependencies, method: &str, input: &Source){
    let supply:u128;
    let supply_str = read_db(TOTAL);
    // event(method, key.as_str());

    if supply_str.len() == 0 {
        supply = 0;
    }else {
        supply = parse_u128(supply_str.as_str())
    }

    return_contract(Ok(Response {
        data: supply.to_string().as_bytes(),
    }));
}

//获取token持有者列表hashmap[addr:"exist"]
fn holder_list(deps: &Dependencies, method: &str, input: &Source){
    let holder_list = _get_holder_list();
    let holder_list_v = serde_json::to_vec(&holder_list).unwrap();
    let holder_list_u8 = holder_list_v.as_slice();
    return_contract(Ok(Response {
        data: holder_list_u8,
    }))
}

fn add_to_holder_list(addr: &str) {
    let mut holder_list = _get_holder_list();
    holder_list.insert(String::from(addr), String::from("exist"));
    let holder_list_v = serde_json::to_vec(&holder_list).unwrap();
    let holder_list_u8 = holder_list_v.as_slice();
    runtime::make_dependencies()
        .storage
        .set(HOLDER_LIST.as_bytes(), holder_list_u8)
}

fn _get_holder_list() -> HashMap<String, String>{
    let holder_list = runtime::make_dependencies()
        .storage
        .get(HOLDER_LIST.as_bytes());
    match holder_list {
        None => {
            HashMap::new()
        }
        Some(holder) => {
            serde_json::from_slice(holder.as_slice()).unwrap()
        }
    }
}


fn has_permission(deps: &Dependencies, action: &str) {
    let acl_addr_str = read_db(ACL_ADDR);
    let acl_addr_str = acl_addr_str.as_str();
    if acl_addr_str.len() == 0 {
        return_contract(Err("acl_addr is empty"));
    }
    let acl_addr = Address::from(acl_addr_str);

    let who = deps.api.get_pre_caller();
    let who = who.to_string();
    let who = who.as_str();
    let what = deps.api.self_address();
    let what = what.to_string();
    let what = what.as_str();

    let mut sink = Sink::new(0);
    sink.write_str("has_permission");
    sink.write_str(who);
    sink.write_str(what);
    sink.write_str(action);
    let ret_input = sink.into();

    match deps.api.call_contract(&acl_addr, &ret_input) {
        Some(res) => if String::from_utf8(res).unwrap().as_str() != "true"{
            return_contract(Err("no permissions"))
        }
        None => return_contract(Err("call contract error")),
    }
}

fn parse_u128(source: &str) -> u128 {
    source.parse::<u128>().unwrap()
}

fn read_balance(addr: &str) -> u128 {
    let balance:u128;
    let addr_str = addr_to_lower(addr);
    let key = BALANCE.to_owned() + addr_str.as_str();
    let balance_str = read_db(key.as_str());
    // event(method, key.as_str());

    if balance_str.len() == 0 {
        balance = 0;
    }else {
        balance = parse_u128(balance_str.as_str())
    }
    return balance;
}

fn addr_to_lower(addr: &str) -> String {
    let mut addr_str = addr.to_string();
    addr_str.make_ascii_lowercase();
    return addr_str
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
            let app_address = Address::new(&res).unwrap();
            return (app_address, true)
        }
    }
}