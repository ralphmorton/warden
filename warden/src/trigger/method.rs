use http::{Method, Request};

pub enum MethodTrigger {
  Any,
  Exactly(Method),
  OneOf(Vec<Method>),
  NoneOf(Vec<Method>),
}

impl MethodTrigger {
  pub fn applies<T>(&self, req: &Request<T>) -> bool {
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

