pub mod request;
pub mod response;

use crate::trigger::request::RequestTrigger;
use crate::trigger::response::ResponseTrigger;
use http::{Request, Response};

pub struct Trigger {
    request: RequestTrigger,
    response: Option<ResponseTrigger>,
}

impl Trigger {
    pub fn new(request: RequestTrigger, response: Option<ResponseTrigger>) -> Self {
        Self { request, response }
    }

    pub fn applies_to_req<T>(&self, req: &Request<T>) -> bool {
        if self.response.is_some() {
            return false;
        }

        self.request.applies(req)
    }

    pub fn applies_to_rsp<T, U>(&self, req: &Request<T>, rsp: &Response<U>) -> bool {
        match &self.response {
            None => false,
            Some(r) => r.applies(rsp) && self.request.applies(req),
        }
    }
}
