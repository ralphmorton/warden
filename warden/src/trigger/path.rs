use http::Request;

pub enum PathTrigger {
  Any,
  Exactly(String),
  Contains(String),
  Regex(regex::Regex),
}

impl PathTrigger {
  pub fn applies<T>(&self, req: &Request<T>) -> bool {
      let uri = req.uri();

      match self {
          Self::Any => true,
          Self::Exactly(path) => uri.path() == path.as_str(),
          Self::Contains(path) => uri.path().to_string().contains(path.as_str()),
          Self::Regex(reg) => reg.is_match(uri.path()),
      }
  }
}

#[cfg(test)]
mod tests {
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

