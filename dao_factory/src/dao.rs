extern crate c123chain_cdk as cdk;

use cdk::types::{Address, Response};
use cdk::runtime;
use cdk::codec::{Sink};
use serde::{Deserialize, Serialize};
use serde_json::Error;
use std::collections::HashMap;
use Clone;

//base app name;
const COMMUNITY_APP_NAME: &str = "community_app";
const ACL_APP_NAME: &str = "acl_app";
const VOTING_APP_NAME: &str = "voting_app";
const TOKEN_APP_NAME: &str = "token_app";
const FINANCE_APP_NAME: &str = "finance_app";

//base app code hash;
const ACL_APP_CODE_HASH: &str = "383E2CC4418EFA8267DEBF2C7DFB5B5F";
const VOTING_APP_CODE_HASH: &str = "36BE31B2EC1BEBD647B22374B210C73C";
const TOKEN_APP_CODE_HASH: &str = "FFD30DCCAC0818919276F08BB6546AD1";
const FINANCE_APP_CODE_HASH: &str = "";


//acl method;
const CREATE_PERMISSION_METHOD: &str = "create_permission";

//permission list
//token
const TOKEN_MINT_PERMISSION: &str = "token.mint";
const TOKEN_BURN_PERMISSION: &str = "token.burn";

//acl
const ACL_CREATE_PER_PERMISSION: &str = "acl.create_per";

//voting

//finance

//default per
#[derive(Default, Clone)]
struct AllPermission {
    who: String,
    what: String,
    action: String,
    manager: String,
}

//all contracts default
#[derive(Default)]
struct AllContracts {
    default_contracts: HashMap<String, String>,
    default_permissions: HashMap<String, Vec<AllPermission>>
}
//permission settings
//all contracts that you want to install;
impl AllContracts {
    fn new() -> AllContracts {
        let mut all = AllContracts::default();
        //
        let mut default_contracts: HashMap<String, String> = HashMap::default();
        default_contracts.insert(ACL_APP_NAME.to_string(), ACL_APP_CODE_HASH.to_string());
        default_contracts.insert(VOTING_APP_NAME.to_string(), VOTING_APP_CODE_HASH.to_string());
        default_contracts.insert(TOKEN_APP_NAME.to_string(), TOKEN_APP_CODE_HASH.to_string());
        default_contracts.insert(FINANCE_APP_NAME.to_string(), FINANCE_APP_CODE_HASH.to_string());
        all.default_contracts = default_contracts;

        //all_acl
        let mut all_acl_per: Vec<AllPermission> = Vec::default();
        let mut acl_per = AllPermission::default();
        acl_per.who = COMMUNITY_APP_NAME.to_string();
        acl_per.what = ACL_APP_NAME.to_string();
        acl_per.action = ACL_CREATE_PER_PERMISSION.to_string();
        acl_per.manager = COMMUNITY_APP_NAME.to_string();
        all_acl_per.push(acl_per);

        //all_token
        let mut all_token_per: Vec<AllPermission> = Vec::default();
        let mut token_per = AllPermission::default();
        token_per.who = VOTING_APP_NAME.to_string();
        token_per.what = TOKEN_APP_NAME.to_string();
        token_per.action = TOKEN_MINT_PERMISSION.to_string();
        token_per.manager = VOTING_APP_NAME.to_string();
        all_token_per.push(token_per);

        let mut token_second_per = AllPermission::default();
        token_second_per.who = VOTING_APP_NAME.to_string();
        token_second_per.what = TOKEN_APP_NAME.to_string();
        token_second_per.action = TOKEN_BURN_PERMISSION.to_string();
        token_second_per.manager = VOTING_APP_NAME.to_string();
        all_token_per.push(token_second_per);

        //default per list
        let mut default_permissions_list: HashMap<String, Vec<AllPermission>> = HashMap::default();
        default_permissions_list.insert(ACL_APP_NAME.to_string(), all_acl_per);
        default_permissions_list.insert(TOKEN_APP_NAME.to_string(), all_token_per);
        all.default_permissions = default_permissions_list;

        return all
    }
}

#[derive(Serialize, Deserialize, Default)]
struct ContractArgs {
    app_name: String,
    init_args: Vec<String>
}

#[derive(Serialize, Deserialize, Default)]
struct Params {
    init_apps: Vec<ContractArgs>
}

#[derive(Serialize, Deserialize, Default)]
struct CallParams {
    method: String,
    args: Vec<String>
}


#[no_mangle]
pub fn create_dao(args: &str) -> (bool, Vec<u8>) {
    let (ok, result) = install_base_app(args);
    return (ok, result)
}

fn install_base_app(apps_args: &str) -> (bool, Vec<u8>) {
    let result: Vec<u8> = Vec::new();
    let mut all_installed_apps: HashMap<String, String> = HashMap::new();
    //get template contract address
    let _community_app_address = _contract_address();
    all_installed_apps.insert(COMMUNITY_APP_NAME.to_string(), _community_app_address.to_string());
    let all_per_list = AllContracts::new();
    
    //set acl permission;
    let mut a = CallParams::default();
    a.args.insert(0, _community_app_address.to_string());
    let acl_input = serde_json::to_vec(&a).unwrap();
    let _acl_app_address = new(ACL_APP_CODE_HASH.as_bytes(), acl_input);
    all_installed_apps.insert(ACL_APP_NAME.to_string(), _acl_app_address.to_string());
    //install base app and initiate.
    let args: Result<Params, Error> = serde_json::from_str(apps_args);
    let params: Params;
    match args {
        Err(_) => {
            return_contract(Err("invalid apps args"));
            return (false, result);
        }
        _ => {
            params= args.unwrap();
            let contracts = &params.init_apps;
            for app in contracts {
                let mut c = CallParams::default();
                c.args.push(_acl_app_address.to_string());
                c.args.push(_community_app_address.to_string());
                for i in &app.init_args {
                    let v = i.clone();
                    c.args.push(v);
                }
                let input = serde_json::to_vec(&c).unwrap();//sink.into();
                let app_hash_str = all_per_list.default_contracts.get(&app.app_name);
                let app_hash;
                match app_hash_str {
                    None => {
                        let ret = &(app.app_name.to_owned() + "---get hash failed, this app not include in dao yet");
                        return_contract(Err(ret));
                        return (false, result);
                    }
                    Some(value) => {
                        app_hash = value;
                    }
                }
                let app_address = new(app_hash.as_bytes(), input);
                
                all_installed_apps.insert(app.app_name.clone(), app_address.to_string());
            }
        }
    }
    //set base permission.
    //return all installed apps map.
    let ret = serde_json::to_vec(&all_installed_apps).unwrap();
    if params.init_apps.len() == 0 {
        return (true, ret);
    }
    for app in &params.init_apps {
        let app_name = &app.app_name;
        let acl_per_list;
        let acl_per_list_op = all_per_list.default_permissions.get(app_name);
        match acl_per_list_op {
            None => {
                //do nothing
                continue;
            }
            Some(value) => {
                acl_per_list = value
            }
        }
        for v in acl_per_list {
            let who;
            let what;
            let manager;
            //set acl base permission
            let who_option = all_installed_apps.get(&*v.who);
            match who_option {
                None => {
                    let ret = &(app_name.to_owned() + "not installed yet");
                    return_contract(Err(ret));
                    return (false, result);
                }
                Some(value) => {
                    who = value
                }
            }
            let what_option = all_installed_apps.get(&*v.what);
            match what_option{
                None => {
                    let ret = &(app_name.to_owned() + "not installed yet");
                    return_contract(Err(ret));
                    return (false, result);
                }
                Some(value) => {
                    what = value
                }
            }
            let manager_option = all_installed_apps.get(&*v.manager);
            match manager_option {
                None => {
                    let ret = &(app_name.to_owned() + "not installed yet");
                    return_contract(Err(ret));
                    return (false, result);
                }
                Some(value) => {
                    manager = value
                }
            }
            let install_acl_ok = set_permission(&_acl_app_address, who, what, &(*v.action), manager);
            if !install_acl_ok {
                return_contract(Err("set acl permission failed"));
                return (false, result);
            }
        }
    }

    return (true, ret)
}

fn new(hash:&[u8], args: Vec<u8>) -> Address {
    //调用go的方法，获得地址
    return runtime::make_dependencies().api.new_contract(hash, &args);
}


fn set_permission(acl: &Address, who: &str, what: &str, action: &str, manager: &str) -> bool {
    let deps = runtime::make_dependencies();
    let mut acl_sink = Sink::new(0);
    acl_sink.write_str(CREATE_PERMISSION_METHOD);
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

fn return_contract<'a>(result: Result<Response, &'a str>) {
    runtime::make_dependencies().api.ret(result)
}

fn _contract_address() -> Address {
    let deps = runtime::make_dependencies();
    let app = deps.api.self_address();
    return app
}