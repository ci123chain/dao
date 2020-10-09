extern crate c123chain_cdk as cdk;
extern crate dao_factory as dao;
extern crate serde_json;

use dao::dao::create_dao;
use cdk::types::{Address, Response};
use cdk::runtime;
use cdk::codec::{Sink, Source};
//use serde_json::Error;
use std::collections::HashMap;
//use cdk::debug;
//const ADDR_SIZE: usize = 20;
//const EMPTY_ENTITY: [u8; ADDR_SIZE] = [0; 20];

const KEY_ALL_APPS: &str = "all_apps";
const KEY_ACL_APP: &str = "acl_app";
const KEY_TOKEN_APP: &str = "token_app";
const KEY_VOTING_APP: &str = "voting_app";
/*
const KEY_INITIAL_CONTRACT: &str = "_initial_contract";
const KEY_DEFAULT_PERMISSION: &str = "default_permissions";
const KEY_SET_CONTRACT_ADDRESS: &str = "set_contract_address";
const KEY_IS_OFFICIAL: &str = "is_official";
*/

const ACTION_MANAGE: &str = "community.manage";
const ACTION_ADD_APP: &str = "community.add_app";
const ACTION_REMOVE_APP: &str = "community.remove_app";

#[no_mangle]
pub fn init() {
    let deps = runtime::make_dependencies();
    let input = deps.api.input();
    let input = Source::new(&input);
    /*
    let official: bool;
    let is_official_str = input.read_str().unwrap();
    match is_official_str {
        "true" => {
            official = true
        }
        "false" => {
            official = false
        }
        _ => {
            return_contract(Err("invalid param is_official"));
            return;
        }
    }
    let mut res_sink = Sink::new(0);
    res_sink.write_bool(false);
    set_param(KEY_INITIAL_CONTRACT, &res_sink.into());
    let mut official_sink = Sink::new(0);
    official_sink.write_bool(official);
    set_param(KEY_IS_OFFICIAL, &official_sink.into());
    */
    let args = input.read_str().unwrap();
    let (ok, ret) = create_dao(args);
    if !ok {
        return;
    }
    set_param(KEY_ALL_APPS, &ret);
    return_contract(Ok(Response { data: "Success".as_bytes()}));
    
}

fn store_apps(map:HashMap<&str, &str>) {
    let res = serde_json::to_vec(&map).unwrap();
    set_param(KEY_ALL_APPS, &res);
}

fn get_apps() -> (Vec<u8>, bool) {
    let (apps, ok) = get_param(KEY_ALL_APPS);
    if !ok {
        return (Vec::new(), false)
    }
    return (apps, true)
}

fn get_app_address(app_name: &str) -> (String, bool) {
    let (app, ok) = get_apps();
    if !ok {
        let res_return = app_name.to_string() + "not existed";
        return (res_return, false)
    }
    let ret = serde_json::from_slice(&app);
    match ret {
        Ok(res) => {
            let app_map: HashMap<&str, &str> = res;
            let addr = app_map.get(app_name);
            match addr {
                Some(value) => {
                    let res_return = value.to_string().clone();
                    return (res_return, true)
                }
                None => {
                    let res_return = app_name.to_string() + "not existed";
                    return (res_return, false)
                }
            }
        }
        Err(_) => {
            return ("parse to hash map failed".to_string(), false)
        }
    }
}

fn add_app(app_name:&str, app_addr:&str) {
    let (app, ok) = get_apps();
    let mut app_map: HashMap<&str, &str> = HashMap::new();
    if ok {
        app_map = serde_json::from_slice(&app).unwrap();
    }
    app_map.insert(app_name, app_addr);
    store_apps(app_map);
    
}

fn remove_app(app_name:&str) -> bool {
    let (app, ok) = get_apps();
    if !ok {
        return_contract(Err("no apps yet"));
        return false
    }
    let mut app_map: HashMap<&str, &str> = serde_json::from_slice(&app).unwrap();
    app_map.remove(&app_name);
    store_apps(app_map);
    return true
}

fn query_balances() {
    let (addr, ok) = get_app_address(KEY_TOKEN_APP);
    if !ok {
        return_contract(Err(&addr));
    }
    let app_address = Address::from(addr.as_str());
    let deps = runtime::make_dependencies();
    let mut sink = Sink::new(0);
    sink.write_str("query_balances");
    let input = sink.into();
    match deps.api.call_contract(&app_address, &input) {
        Some(res) => {
            return_contract(Ok(Response{ data: &res}));
        },
        None => {
            return_contract(Err("query balance failed, call token contract error, return none"));
        }
    }
}

fn query_votes() {
    let (addr, ok) = get_app_address(KEY_VOTING_APP);
    if !ok {
        return_contract(Err(&addr));
    }
    let app_address = Address::from(addr.as_str());
    let deps = runtime::make_dependencies();
    let mut sink = Sink::new(0);
    sink.write_str("query_votes");
    let input = sink.into();
    match deps.api.call_contract(&app_address, &input) {
        Some(res) => {
            return_contract(Ok(Response{ data: &res}));
        },
        None => {
            return_contract(Err("query votes failed,call voting contract error, return none"));
        }
    }
}

fn query_permissions() {
    let (addr, ok) = get_app_address(KEY_ACL_APP);
    if !ok {
        return_contract(Err(&addr));
    }
    let app_address = Address::from(addr.as_str());
    let deps = runtime::make_dependencies();
    let mut sink = Sink::new(0);
    sink.write_str("query_permissions");
    let input = sink.into();
    match deps.api.call_contract(&app_address, &input) {
        Some(res) => {
            return_contract(Ok(Response{ data: &res}));
        },
        None => {
            return_contract(Err("query permissions failed, call acl contract error, return none"));
        }
    }
}

/*
fn set_default_permission(apps: &[&Address], acl_address: Address) -> bool {
    for app in apps {
        let mut sink = Sink::new(0);
        sink.write_str(KEY_DEFAULT_PERMISSION);
        let (default_per_list, ok) = get_default_permissions(*app, &sink.into());
        if !ok {
            return false;
        }
        for (k, v ) in default_per_list.iter() {
            let result = set_acl_permission(&acl_address, v, &(*(app.to_string())), k, v);
            if !result{
                return false;
            }
        }
    }
    return true
}

fn set_contract_address(token_address: Address, voting_address:Address) -> bool {
    //set contract address;
    let deps = runtime::make_dependencies();
    let mut sink = Sink::new(0);
    sink.write_str(KEY_SET_CONTRACT_ADDRESS);
    sink.write_str(&(*(token_address.to_string())));
    let input = sink.into();
    match deps.api.call_contract(&voting_address, &input) {
        Some(res) => {
            let ret = String::from_utf8(res);
            match ret {
                Ok(response) => {
                    let res_str = &(*response);
                    match res_str {
                        "Success" => {
                            return true;
                        }
                        _ => {
                            return_contract(Err("call voting contract error, set contract address in voting failed"));
                            return false;
                        }
                    }
                }
                Err(_) => {
                    return_contract(Err("call voting contract error, failed to parse response of set contract address"));
                    return false;
                }
            }
        },
        None => {
            return_contract(Err("call voting contract error, return none"));
            return false;
        },
    }
}
*/

#[no_mangle]
pub fn invoke() {
    let deps = runtime::make_dependencies();
    let input = deps.api.input();
    let input = Source::new(&input);
    let method = input.read_str().unwrap();
    match method {
        
        /*
        "initial_contract" => {
            let (data, _) = get_param(KEY_INITIAL_CONTRACT);
            let source = Source::new(&data);
            let has_set = source.read_bool().unwrap();
            if has_set {
                return_contract(Err("initial contract has been execute already"));
                return;
            }
            let acl_app_address = input.read_str().unwrap();
            let voting_app_address = input.read_str().unwrap();
            let acl_address = Address::from(acl_app_address);
            let voting_address = Address::from(voting_app_address);

            let mut app_list: Vec<&Address> = vec![&acl_address, &voting_address];
            let mut map:HashMap<&str, &str> = HashMap::new();
            map.insert(KEY_ACL_APP, acl_app_address);
            map.insert(KEY_VOTING_APP, voting_app_address);
            let mut token_address: Address = Address::new(&EMPTY_ENTITY).unwrap();
            if !is_official() {
                let token_app_address = input.read_str().unwrap();
                token_address = Address::from(token_app_address);
                app_list.push(&token_address);
                map.insert(KEY_TOKEN_APP, token_app_address);
            }

            let result = set_default_permission(&*app_list, acl_address);
            //let result = set_acl_initial_permission_and_contract_address(&voting_address, &token_address, &acl_address);
            if !result {
                return;
            }
            let mut self_map = default_permission();
            self_map.insert(String::from(ACTION_ADD_APP), voting_address.to_string());
            self_map.insert(String::from(ACTION_REMOVE_APP), voting_address.to_string());
            self_map.insert(String::from(ACTION_MANAGE), voting_address.to_string());
            for (k, v) in self_map.iter() {
                let com_addr = &(*(_contract_address().to_string()));
                let result = set_acl_permission(&acl_address, v, com_addr, k, v);
                if !result{
                    return;
                }
            }
            if !is_official() {
                let ok = set_contract_address(token_address, voting_address);
                if !ok {
                    return;
                }
            }
            //save apps;
            store_apps(map);

            let mut res_sink = Sink::new(0);
            res_sink.write_bool(true);
            set_param(KEY_INITIAL_CONTRACT, &res_sink.into());
            return_contract(Ok(Response { data: "Success".as_bytes()}));
        }
        */

        "add_app" => {
            let who = &(_pre_caller().to_string());
            let what = &(_contract_address().to_string());
            let action = ACTION_ADD_APP;
            let ok = has_permission(who, what, action);
            if !ok {
                return;
            }
            let app_name = input.read_str().unwrap();
            let address = input.read_str().unwrap();
            add_app(app_name, address);
            return_contract(Ok(Response{ data: "Success".as_bytes()}));
        }

        "remove_app" => {
            let who = &(_pre_caller().to_string());
            let what = &(_contract_address().to_string());
            let action = ACTION_REMOVE_APP;
            let ok = has_permission(who, what, action);
            if !ok {
                return;
            }
            let app_name = input.read_str().unwrap();
            let ok = remove_app(app_name);
            if !ok {
                return;
            }
            return_contract(Ok(Response{ data: "Success".as_bytes()}));
        }

        "default_permissions" => {
            let map = default_permission();
            let res = serde_json::to_vec(&map).unwrap();
            return_contract(Ok(Response {data: &res}));
        }

        "query_apps" => {
            let (apps, ok) = get_apps();
            if !ok {
                return_contract(Err("no apps exists"));
                return;
            }
            return_contract(Ok(Response{ data: &apps}));
        }
        "query_votes" => {
            query_votes();
        }

        "query_balances" => {
            query_balances();
        }

        "query_app" => {
            let name = input.read_str().unwrap();
            let (address, ok) = get_app_address(name);
            if !ok {
                return_contract(Err(&address));
                return;
            }
            return_contract(Ok(Response{ data: address.as_bytes()}));
        }

        "query_permissions" => {
            query_permissions()
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
        "query_apps" => {
            let (apps, ok) = get_apps();
            if !ok {
                return_contract(Err("no apps exists"));
                return;
            }
            return_contract(Ok(Response{ data: &apps}));
        }
        "query_votes" => {
            query_votes();
        }

        "query_balances" => {
            query_balances();
        }

        "query_app" => {
            let name = input.read_str().unwrap();
            let (address, ok) = get_app_address(name);
            if !ok {
                return_contract(Err(&address));
                return;
            }
            return_contract(Ok(Response{ data: address.as_bytes()}));
        }

        "query_permissions" => {
            query_permissions()
        }

        _ => {
            // 返回Error
            return_contract(Err("invoke method not found"));
        }
    }
}

/*
fn set_acl_permission(acl: &Address, who: &str, what: &str, action: &str, manager: &str) -> bool {
    let deps = runtime::make_dependencies();
    let mut acl_sink = Sink::new(0);
    acl_sink.write_str("create_permission");
    acl_sink.write_str(who);
    acl_sink.write_str(what);
    acl_sink.write_str(action);
    acl_sink.write_str(manager);
    let acl_input = acl_sink.into();
    match deps.api.call_contract(acl, &acl_input) {
        Some(res) => {
            let ret = String::from_utf8(res);
            match ret {
                Ok(response) => {
                    let result = &(*response);
                    match result {
                        "success" => {
                            return true;
                        }
                        _ => {
                            return_contract(Err("call acl contract error, set permission failed"));
                            return false;
                        }
                    }
                }
                Err(_) => {
                    return_contract(Err("call acl contract error, failed to parse response of create default permission"));
                    return false;
                }
            }
        },
        None => {
            return_contract(Err("call contract error, no response"));
            return false;
        },
    };
}
*/

fn default_permission() -> HashMap<String, String> {
    let app_address = _contract_address();
    let mut map: HashMap<String, String> = HashMap::new();
    map.insert(String::from(ACTION_MANAGE), app_address.to_string());
    return map
}

fn has_permission(who: &str, what: &str, action: &str) -> bool {
    let (acl_address, ok) = get_app_address(KEY_ACL_APP);
    if !ok {
        return_contract(Err(&acl_address));
    }
    let deps = runtime::make_dependencies();
    let acl = Address::from(acl_address.as_str());
    let mut  acl_sink = Sink::new(0);
    acl_sink.write_str("has_permission");
    acl_sink.write_str(who);
    acl_sink.write_str(what);
    acl_sink.write_str(action);
    let acl_input = acl_sink.into();
    match deps.api.call_contract(&acl, &acl_input) {
        Some(res) => {
            let ret = String::from_utf8(res);
            match ret {
                Ok(response) => {
                    let result = &(*response);
                    match result {
                        "true" => {
                            return true;
                        }
                        _ => {
                            return_contract(Err("call acl contract error, you have no permission"));
                            return false;
                        }
                    }
                }
                Err(_) => {
                    return_contract(Err("call acl contract error, failed to parse response of has permission"));
                    return false;
                }
            }
        },
        None => {
            return_contract(Err("call contract error, no response"));
            return false;
        },
    };
}

/*
fn get_default_permissions(contract_address: &Address, input: &[u8]) ->  (HashMap<String,String>, bool) {
    let deps = runtime::make_dependencies();
    let empty_map: HashMap<String, String> = HashMap::new();
    match deps.api.call_contract(contract_address, input) {
        Some(res) => {
            let result = res.clone();
            let ret: Result<HashMap<String, String>, Error> = serde_json::from_slice(&result);
            match ret {
                Ok(response) => {
                    return (response, true)
                }
                Err(_) => {
                    let resp = "call contract ".to_owned() + &(*(contract_address.to_string())) + " error, failed to parse default permission list";
                    return_contract(Err(&resp));
                    return (empty_map, false);
                }
            }
        },
        None => {
            let resp = "call contract ".to_owned() + &(*(contract_address.to_string())) + "error, got no default permission list";
            return_contract(Err(&resp));
            return (empty_map, false);
        }
    }
}
*/

fn _contract_address() -> Address {
    let app = runtime::make_dependencies().api.self_address();
    return app
}

fn _get_invoker() -> Address {
    let app = runtime::make_dependencies().api.get_invoker();
    return app
}
fn _pre_caller() -> Address {
    let app = runtime::make_dependencies().api.get_pre_caller();
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

fn return_contract<'a>(result: Result<Response, &'a str>) {
    runtime::make_dependencies().api.ret(result)
}

/*
fn is_official() -> bool {
    let (res, _) = get_param(KEY_IS_OFFICIAL);
    let source = Source::new(&res);
    let resp = source.read_bool().unwrap();
    return resp
}
*/


