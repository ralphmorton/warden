use crate::args::Args;
use crate::action::Action;
use crate::action::proxy::Proxy;
use crate::trigger::Trigger;
use http::Request;
use std::sync::Arc;

pub type Ruleset = Arc<Vec<Rule>>;

pub struct Rule {
    trigger: Trigger,
    action: Action
}

impl Rule {
    pub fn applies<T>(&self, req: &Request<T>) -> bool {
        self.trigger.applies(req)
    }

    pub fn transform_req<T>(&self, req: Request<T>) -> Option<Request<T>> {
        self.action.transform_req(req)
    }
}

pub fn start(args: &Args) -> Ruleset {
    let default_proxy = Proxy::builder()
        .scheme(args.downstream_scheme.clone())
        .host(args.downstream_host.clone());

    let default_proxy = match args.downstream_port {
        Some(port) => default_proxy.port(port),
        None => default_proxy,
    };

    let default_proxy = default_proxy.build().expect("default downstream proxy is valid");

    let default_rule = Rule {
        trigger: Trigger::catch_all(),
        action: Action::Proxy(default_proxy),
    };

    Arc::new(vec![default_rule])
}
