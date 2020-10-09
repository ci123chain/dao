extern crate c123chain_cdk as cdk;
extern crate serde_json;
extern crate serde;
extern crate sha2;

use std::collections::HashMap;
use cdk::debug;
use cdk::runtime;
use cdk::types::{Address, Response, ADDR_SIZE};
use cdk::codec::Source;
use sha2::{Sha256, Digest};

const ACTION_PERM_CREATE: &str = "acl.createperm";

// 权限点 key
const MANAGER_KEY: &[u8] = "manager".as_bytes();

const COMMUNITY_KEY: &[u8] = "community".as_bytes();

const PERMISSION_KEY: &[u8] = "permissions".as_bytes();

const PERMISSION_OK: u8 = 1;
// const PERMISSION_REJECTED: u8 = 0;
// const ACTION_PERM_MANAGE: &str = "acl.manageperm";

const ANY_ENTITY: [u8; ADDR_SIZE] = [1; 20];

#[no_mangle]
pub fn init() {
    let deps = runtime::make_dependencies();
    let input_v = deps.api.input();
    let input = Source::new(&input_v);
    // 设置社区地址
    let a = input.read_str().unwrap();
    let community_address = Address::from(a);
    _set_community_address(&community_address);

    // 设置 community 和 合约创建者 有 acl.manageperm 权限
    // _set_permission(&pre_caller(), &_contract_address(), ACTION_PERM_CREATE);
    _set_permission(&community_address, &_contract_address(), ACTION_PERM_CREATE);
}

#[no_mangle]
pub fn invoke() {
    let deps = runtime::make_dependencies();
    let input_v = deps.api.input();
    let input = Source::new(&input_v);
    let method = input.read_str().unwrap();
    match method {
        "inittest" => {
            debug!("testing aaaaa");
        }

        "has_permission" => {
            let who = Address::from(input.read_str().unwrap());
            let what = Address::from(input.read_str().unwrap());
            let action = input.read_str().unwrap();
            let has_permission = if has_permission(&who, &what, action) {"true"} else {"false"};
            return_contract(Ok(Response{data: has_permission.as_bytes()}))
        }

        "create_permission" => {
            let who = Address::from(input.read_str().unwrap());
            let what = Address::from(input.read_str().unwrap());
            let action = input.read_str().unwrap();
            let manager = Address::from(input.read_str().unwrap());
            let selfs = deps.api.self_address();

            // 判断是否有权限 创建权限
            if has_permission(&pre_caller(), &selfs, ACTION_PERM_CREATE) {
                create_permission(who, what, action, manager);
                return_contract(Ok(Response{data: "success".as_bytes()}))
            } else {
                return_contract(Err("you don't have permission to create"));
            }
        }

        "grant_permission" => {
            let who = Address::from(input.read_str().unwrap());
            let what = Address::from(input.read_str().unwrap());
            let action = input.read_str().unwrap();
            // 只有权限管理者可以分配权限给 实体
            if _is_permission_manager(&pre_caller(), &what, action)  {
                grant_permission(who, what, action);
            } else {
                return_contract(Err("you don't have permission to grant"));
            }
        }

        "revoke_permission" => {
            let who = Address::from(input.read_str().unwrap());
            let what = Address::from(input.read_str().unwrap());
            let action = input.read_str().unwrap();
            // 只有权限管理者可以销毁权限
            if _is_permission_manager(&pre_caller(), &what, action) {
                revoke_permission(who, what, action);
            } else {
                return_contract(Err("you don't have permission to revoke"));
            }
        }

        "set_permission_manager" => {
            let what = Address::from(input.read_str().unwrap());
            let action = input.read_str().unwrap();
            let new_mgr = Address::from(input.read_str().unwrap());

            // 判断是否有权限 销毁权限
            if !_is_permission_manager(&pre_caller(), &what, action) {
                return_contract(Err("you don't have permission to set manager"));
                return
            }
            set_permission_manager(&what, action, new_mgr);            
        }

        "delete_permission_manager" => {
            let what = Address::from(input.read_str().unwrap());
            let action = input.read_str().unwrap();

             // 判断是否是自己
             if !_is_permission_manager(&pre_caller(), &what, action) {
                return_contract(Err("you don't have permission to delete manager"));
                return
             }
            delete_permission_manager(&what, action);
        }

        "default_permissions" => {
            default_permissions();
        }
        _ => {
            // 返回Error
            return_contract(Err("method not found"));
        }
    }
}


// subscribe 基础用法 query = "type.key = 'value'"
// fn event(method: &str, msg: &str) {
//     let mut event = runtime::Event::new(method);
//     event.add("msg", IString(msg));
//     runtime::make_dependencies().api.notify(&event);
// }

fn return_contract<'a>(result: Result<Response, &'a str>) {
    runtime::make_dependencies().api.ret(result)
}

// 创建权限
fn create_permission(who: Address, what: Address, action: &str, manager: Address) {
    _set_permission_manager(&manager, &what, action);  
    _set_permission(&who, &what, action);
    // permissions[permissionHash(_entity, _app, _role)] = _paramsHash;
}   

// 通过角色权限
fn has_permission(who: &Address, what: &Address, action: &str) -> bool {
    let permissions = _recover_permissions();
    let hash_key = _permission_hash(who, what, action);
    match permissions.get(hash_key.as_str()) {
        None => {
            let any_hash_key = _permission_hash(&Address::new(&ANY_ENTITY).unwrap(), what, action);
            match permissions.get(any_hash_key.as_str()) {
                None => { return false }
                Some(_) => { return true }
            }
        }
        Some(_) => {
            true
        }
    }
}


// 通过角色权限
fn grant_permission(who: Address, what: Address, action: &str) {
    _set_permission(&who, &what, action);
}

// 销毁角色权限
fn revoke_permission(who: Address, what: Address, action: &str) {
    // todo filter manager  
    let mut permissions = _recover_permissions();
    let hash_key = _permission_hash(&who, &what, action);
    
    let res = permissions.remove(&hash_key);
    match res {
        Some(_) => {
            _save_permissions(permissions);
        }
        None => {
            return_contract(Err("permission not exist"))
        }
    }
}

// 设置角色权限管理者
fn set_permission_manager(what: &Address, action: &str, manager: Address) {
    
    _set_permission_manager(&manager, &what, action);
}

// 删除角色权限管理者
fn delete_permission_manager(what: &Address, action: &str) {
    _delete_permission_manager(&what, action);
}




// 是否是权限管理者
fn _delete_permission_manager(what: &Address, action: &str) {
    let mut per_manager_map = _recover_permission_manager();
    let key = _action_hash(what, action);
    per_manager_map.remove(key.as_str());
    _save_permission_manager(per_manager_map);
}

fn _is_permission_manager(mgr: &Address, what: &Address, action: &str) -> bool {
    match _get_permission_manager(what, action) {
        None => {
            false
        }
        Some(addr) => {
            if &addr == mgr {
                return true
            }
            false
        }
    } 
}

fn _set_permission(who: &Address, what: &Address, action: &str) {
    let mut permissions = _recover_permissions();
    let hex_key = _permission_hash(who, what, action);
    permissions.insert(hex_key, PERMISSION_OK);
    _save_permissions(permissions);
} 

fn _set_permission_list(who: &Address, what: &Address, actions: &[&str]) {
    let mut permissions = _recover_permissions();
    for action in actions.iter() {
        let hex_key = _permission_hash(who, what, action);
        permissions.insert(hex_key, PERMISSION_OK);
    }
    _save_permissions(permissions);
} 

// 获取所有权限 map
fn _recover_permissions() -> HashMap<String, u8> {
    let permission_v = runtime::make_dependencies()
        .storage
        .get(PERMISSION_KEY);
    match permission_v {
        None => {
            let a = HashMap::new();
            return a;
        }
        Some(permission) => {
            let a = serde_json::from_slice(&permission).unwrap();
            return a;
        }
    }
}

/// --------- 权限列表 ---------

// 保存所有权限列表 map
fn _save_permissions(permissions: HashMap<String, u8>) {
    let permission_v = serde_json::to_vec(&permissions).unwrap();
    let permission_u8 = permission_v.as_slice();
    runtime::make_dependencies()
        .storage
        .set(PERMISSION_KEY, permission_u8)
}


/// --------- 权限管理者 ---------

fn _recover_permission_manager() -> HashMap<String, Address> {
    let permission_manger_v = runtime::make_dependencies()
        .storage
        .get(MANAGER_KEY);
    match permission_manger_v {
        None => {
            HashMap::new()
        }
        Some(permission_manager) => {
            //debug!("{:#?}", permission_manager);
            let b: HashMap<String, Address>;
            b = serde_json::from_slice(permission_manager.as_slice()).unwrap();
            return b;
            //HashMap::new()
        }
    }
}

fn _get_permission_manager(what: &Address, action: &str) -> Option<Address> {
    let per_manager_map = _recover_permission_manager();
    let key = _action_hash(what, action);
    
    let value = per_manager_map.get(key.as_str());
    match value {
        Some(v) => {
            Some(*v)
        }
        None => {
            None
        }
    }
}

fn _set_permission_manager(manager: &Address, what: &Address, action: &str) {
    let mut per_manager_map = _recover_permission_manager();

    let key = _action_hash(what, action);

    per_manager_map.insert(key, manager.clone());
    _save_permission_manager(per_manager_map);
}

fn _save_permission_manager(per_manager: HashMap<String, Address>) {
    let permission_v = serde_json::to_vec(&per_manager).unwrap();
    let permission_u8 = permission_v.as_slice();
    runtime::make_dependencies()
        .storage
        .set(MANAGER_KEY, permission_u8)
}



/// --------- 社区地址 ---------

fn _set_community_address(community: &Address) {
    runtime::make_dependencies()
        .storage
        .set(COMMUNITY_KEY, community.as_bytes());
}

fn _get_community_address() -> Option<Address> {
    let community_address = runtime::make_dependencies()
        .storage
        .get(COMMUNITY_KEY);
        
        match community_address {
            None => {
                None
            }
            Some(community) => {
                Address::new(community.as_slice())
            }
        }    
}


/// --------- Hash 计算 ---------

fn _permission_hash(entity: &Address, app: &Address, action: &str) -> String {
    let mut key = String::new();                     
    key.push_str("permissions");

    let entitys = entity.to_string();
    key.push_str(entitys.as_str());

    let apps = app.to_string();
    key.push_str(apps.as_str());
    

    key.push_str(action);
    
    // create a Sha256 object
    let mut hasher = Sha256::new();
    // write input message
    hasher.update(key.as_str());
    // read hash digest
    let hex_key = hasher.finalize();

    return String::from_utf8_lossy(&hex_key[..]).into_owned();
}

fn _action_hash(what: &Address, action: &str) -> String {
    let mut key = String::new();                     
    key.push_str("actions");

    let whats = what.to_string();
    key.push_str(whats.as_str());

    key.push_str(action);

    // create a Sha256 object
    let mut hasher = Sha256::new();
    // write input message
    hasher.update(key.as_str());
    // read hash digest
    let hex_key = hasher.finalize();

    return String::from_utf8_lossy(&hex_key[..]).into_owned();
}


/// --------- Util ---------

fn _contract_address() -> Address {
    let deps = runtime::make_dependencies();
    deps.api.self_address()
}


fn pre_caller() -> Address {
    let deps = runtime::make_dependencies();
    return deps.api.get_pre_caller()
}


// return: key -> value
// key : action
// value: entity
// manager 与 entity 相同
fn default_permissions() {
    let mut default_role_perm: HashMap<String, String> = HashMap::new();
    let c_addr = _contract_address();
    default_role_perm.insert(String::from(ACTION_PERM_CREATE), c_addr.to_string());

    let default_role_perm_v = serde_json::to_vec(&default_role_perm).unwrap();
    let default_role = default_role_perm_v.as_slice();
    return_contract(Ok(Response{data: default_role}));
}

// fn read_string_pairs(input: Source) -> HashMap<String, Address> {
//     let mut perm_map = HashMap::new();

//     while !input.is_eof() {
//         let mut key: String = String::from("");
//         let mut value: Address = Address::zero();
//         match input.read_str() {
//             Err(_) => {
//                 return_contract(Err("paramter unexpect error "));
//             }
//             Ok(str) => {
//                 key = String::from(str);
//             }
//         }

//         match input.read_str() {
//             Err(_) => {
//                 return_contract(Err("paramter unexpect error "));
//             }
//             Ok(str) => {
//                 value = Address::from(str);
//             }
//         }
//         perm_map.insert(key, value);
//     }
//     return perm_map;
// }
