pub mod method;
pub mod path;

use crate::trigger::method::MethodTrigger;
use crate::trigger::path::PathTrigger;
use http::Request;

pub struct Trigger {
    path: PathTrigger,
    method: MethodTrigger,
}

impl Trigger {
    pub fn new(path: PathTrigger, method: MethodTrigger) -> Self {
        Self { path, method }
    }

    pub fn catch_all() -> Self {
        Self {
            path: PathTrigger::Any,
            method: MethodTrigger::Any,
        }
    }

    pub fn applies<T>(&self, req: &Request<T>) -> bool {
        self.path.applies(req) && self.method.applies(req)
    }
}
