use http::{Method, Request};

pub struct RequestTrigger {
    path: PathTrigger,
    method: MethodTrigger,
}

impl RequestTrigger {
    pub fn new(path: PathTrigger, method: MethodTrigger) -> Self {
        Self { path, method }
    }

    pub fn applies<T>(&self, req: &Request<T>) -> bool {
        self.path.applies(req) && self.method.applies(req)
    }
}

pub enum PathTrigger {
    Any,
    Exactly(String),
    Contains(String),
    Regex(regex::Regex),
}

impl PathTrigger {
    fn applies<T>(&self, req: &Request<T>) -> bool {
        let uri = req.uri();

        match self {
            Self::Any => true,
            Self::Exactly(path) => uri.path() == path.as_str(),
            Self::Contains(path) => uri.path().to_string().contains(path.as_str()),
            Self::Regex(reg) => reg.is_match(uri.path()),
        }
    }
}

pub enum MethodTrigger {
    Any,
    Exactly(Method),
    OneOf(Vec<Method>),
    NoneOf(Vec<Method>),
}

impl MethodTrigger {
    fn applies<T>(&self, req: &Request<T>) -> bool {
        match self {
            Self::Any => true,
            Self::Exactly(method) => req.method() == &method,
            Self::OneOf(methods) => methods.iter().find(|m| m == req.method()).is_some(),
            Self::NoneOf(methods) => methods.iter().find(|m| m == req.method()).is_none(),
        }
    }
}

#[cfg(test)]
mod tests {
    mod path {
        #[test]
        fn any_applies() {
            let paths = vec!["", "foo", "foo/bar"];

            for path in paths {
                let uri = http::Uri::builder()
                    .scheme("http")
                    .authority("foo.com")
                    .path_and_query(path)
                    .build()
                    .unwrap();

                let req = http::Request::builder().uri(uri).body(()).unwrap();

                let res = super::super::PathTrigger::Any.applies(&req);
                assert!(res, "PathTrigger::Any applies to any path");
            }
        }

        #[test]
        fn exactly_applies() {
            let uri = http::Uri::builder()
                .scheme("http")
                .authority("foo.com")
                .path_and_query("foo/bar")
                .build()
                .unwrap();

            let req = http::Request::builder().uri(uri).body(()).unwrap();

            let res_y = super::super::PathTrigger::Exactly(String::from("foo/bar")).applies(&req);
            assert!(res_y, "PathTrigger::Exactly applies only to exact matches");

            let res_n = super::super::PathTrigger::Exactly(String::from("foo")).applies(&req);
            assert!(!res_n, "PathTrigger::Exactly applies only to exact matches");
        }

        #[test]
        fn contains_applies() {
            let uri = http::Uri::builder()
                .scheme("http")
                .authority("foo.com")
                .path_and_query("foo/bar")
                .build()
                .unwrap();

            let req = http::Request::builder().uri(uri).body(()).unwrap();

            let res_y = super::super::PathTrigger::Contains(String::from("foo")).applies(&req);
            assert!(res_y, "PathTrigger::Contains applies only to contained");

            let res_n = super::super::PathTrigger::Contains(String::from("baz")).applies(&req);
            assert!(!res_n, "PathTrigger::Contains applies only to contained");
        }

        #[test]
        fn regex_applies() {
            let uri = http::Uri::builder()
                .scheme("http")
                .authority("foo.com")
                .path_and_query("foo/bar/baz")
                .build()
                .unwrap();

            let req = http::Request::builder().uri(uri).body(()).unwrap();

            let reg_y = regex::Regex::new("foo/[a-z]{3}/baz").unwrap();
            let res_y = super::super::PathTrigger::Regex(reg_y).applies(&req);
            assert!(res_y, "PathTrigger::Regex applies only to matching");

            let reg_n = regex::Regex::new("baz/*/foo").unwrap();
            let res_n = super::super::PathTrigger::Regex(reg_n).applies(&req);
            assert!(!res_n, "PathTrigger::Regex applies only to matching");
        }
    }

    mod method {
        #[test]
        fn any_applies() {
            let methods = vec![http::Method::GET, http::Method::POST, http::Method::PUT];

            for method in methods {
                let uri = http::Uri::builder()
                    .scheme("http")
                    .authority("foo.com")
                    .path_and_query("")
                    .build()
                    .unwrap();

                let req = http::Request::builder()
                    .method(method)
                    .uri(uri)
                    .body(())
                    .unwrap();

                let res = super::super::MethodTrigger::Any.applies(&req);
                assert!(res, "MethodTrigger::Any applies to any method");
            }
        }

        #[test]
        fn exactly_applies() {
            let uri = http::Uri::builder()
                .scheme("http")
                .authority("foo.com")
                .path_and_query("")
                .build()
                .unwrap();

            let req = http::Request::builder()
                .method(http::Method::GET)
                .uri(uri)
                .body(())
                .unwrap();

            let res_y = super::super::MethodTrigger::Exactly(http::Method::GET).applies(&req);
            assert!(res_y, "MethodTrigger::Exactly applies only to exact method");

            let res_n = super::super::MethodTrigger::Exactly(http::Method::PUT).applies(&req);
            assert!(
                !res_n,
                "MethodTrigger::Exactly applies only to exact method"
            );
        }

        #[test]
        fn one_of_applies() {
            let uri = http::Uri::builder()
                .scheme("http")
                .authority("foo.com")
                .path_and_query("")
                .build()
                .unwrap();

            let req = http::Request::builder()
                .method(http::Method::GET)
                .uri(uri)
                .body(())
                .unwrap();

            let mx_y = vec![http::Method::GET, http::Method::POST];
            let res_y = super::super::MethodTrigger::OneOf(mx_y).applies(&req);
            assert!(
                res_y,
                "MethodTrigger::OneOf applies only to intersecting methods"
            );

            let mx_n = vec![http::Method::DELETE, http::Method::POST];
            let res_n = super::super::MethodTrigger::OneOf(mx_n).applies(&req);
            assert!(
                !res_n,
                "MethodTrigger::OneOf applies only to intersecting methods"
            );
        }

        #[test]
        fn none_of_applies() {
            let uri = http::Uri::builder()
                .scheme("http")
                .authority("foo.com")
                .path_and_query("")
                .build()
                .unwrap();

            let req = http::Request::builder()
                .method(http::Method::GET)
                .uri(uri)
                .body(())
                .unwrap();

            let mx_y = vec![http::Method::DELETE, http::Method::POST];
            let res_y = super::super::MethodTrigger::NoneOf(mx_y).applies(&req);
            assert!(
                res_y,
                "MethodTrigger::NoneOf applies only to non-intersecting methods"
            );

            let mx_n = vec![http::Method::GET, http::Method::POST];
            let res_n = super::super::MethodTrigger::NoneOf(mx_n).applies(&req);
            assert!(
                !res_n,
                "MethodTrigger::NoneOf applies only to non-intersecting methods"
            );
        }
    }
}
