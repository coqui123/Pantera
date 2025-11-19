use serde::{Deserialize, Serialize};

// Struct to hold admin status
#[derive(Clone, Debug)]
pub struct AdminAuth {
    pub is_dev_admin: bool,
    pub tezos_admin_address: Option<String>,
}

impl AdminAuth {
    pub fn is_admin(&self) -> bool {
        self.is_dev_admin || self.tezos_admin_address.is_some()
    }
    
    /// Create a non-admin auth for public routes
    #[allow(dead_code)]
    pub fn public() -> Self {
        Self {
            is_dev_admin: false,
            tezos_admin_address: None,
        }
    }
}

// Session data for Tezos admin
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TezosAdminSession {
    pub address: String,
}

 