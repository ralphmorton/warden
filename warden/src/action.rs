pub mod proxy;

use crate::action::proxy::Proxy;
use http::Request;

pub enum Action {
    Proxy(Proxy),
}

impl Action {
    pub fn transform_req<T>(&self, req: Request<T>) -> Option<Request<T>> {
        let builder = match self {
            Self::Proxy(proxy) => proxy.transform_req(&req),
        }?;

        builder.body(req.into_body()).ok()
    }
}
