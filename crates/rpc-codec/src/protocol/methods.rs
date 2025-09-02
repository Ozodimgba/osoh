use super::constants::method_ids;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct MethodInfo {
    pub id: u16,
    pub name: &'static str,
    pub description: &'static str,
}

pub struct MethodRegistry {
    by_name: HashMap<&'static str, MethodInfo>,
    by_id: HashMap<u16, MethodInfo>,
}

impl MethodRegistry {
    pub fn new() -> Self {
        let methods = vec![
            MethodInfo {
                id: method_ids::GET_ACCOUNT_INFO,
                name: "getAccountInfo",
                description: "Returns account information for a given public key",
            },
            MethodInfo {
                id: method_ids::GET_BALANCE,
                name: "getBalance",
                description: "Returns the balance of an account",
            },
            MethodInfo {
                id: method_ids::SEND_TRANSACTION,
                name: "sendTransaction",
                description: "Submits a transaction to the network",
            },
            // Add all other methods...
        ];

        let mut by_name = HashMap::new();
        let mut by_id = HashMap::new();

        for method in methods {
            by_name.insert(method.name, method.clone());
            by_id.insert(method.id, method);
        }

        Self { by_name, by_id }
    }

    pub fn get_by_name(&self, name: &str) -> Option<&MethodInfo> {
        self.by_name.get(name)
    }

    pub fn get_by_id(&self, id: u16) -> Option<&MethodInfo> {
        self.by_id.get(&id)
    }

    pub fn all_methods(&self) -> Vec<&MethodInfo> {
        self.by_id.values().collect()
    }
}

impl Default for MethodRegistry {
    fn default() -> Self {
        Self::new()
    }
}
