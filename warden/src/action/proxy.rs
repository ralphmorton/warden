use http::request;
use http::{Request, Uri};

pub struct Proxy {
    scheme: String,
    host: String,
    port: Option<u16>,
    path: Option<PathUpdate>,
    query: Option<QueryUpdate>,
}

impl Proxy {
    pub fn builder() -> Builder {
        Builder::new()
    }

    pub fn transform_req<T>(&self, req: &Request<T>) -> Option<request::Builder> {
        let authority = match self.port {
            Some(p) => format!("{}:{}", self.host, p),
            None => self.host.clone(),
        };

        let path = match &self.path {
            Some(p) => p.apply(req.uri().path()),
            None => req.uri().path().to_string(),
        };

        let query = match &self.query {
            Some(q) => Some(q.apply(req.uri().query())),
            None => req.uri().query().map(String::from)
        };

        let path_and_query = match query {
            Some(q) => format!("{}?{}", path, q),
            None => path,
        };

        let uri = Uri::builder()
            .scheme(self.scheme.as_str())
            .authority(authority.as_str())
            .path_and_query(path_and_query)
            .build()
            .ok()?;

        let mut builder = Request::builder()
            .method(req.method())
            .uri(uri)
            .version(req.version());

        for (key, value) in req.headers().iter() {
            builder = builder.header(key, value);
        }

        Some(builder)
    }
}

pub enum PathUpdate {
    Replace(String),
    Prepend(String),
    Append(String),
}

impl PathUpdate {
    fn apply(&self, existing: &str) -> String {
        match self {
            Self::Replace(p) => p.clone(),
            Self::Prepend(p) => prepend_path(&p, existing),
            Self::Append(p) => prepend_path(existing, &p),
        }
    }
}

fn prepend_path(prefix: &str, rest: &str) -> String {
    format!(
        "{}/{}",
        prefix.strip_suffix("/").unwrap_or(prefix),
        rest.strip_prefix("/").unwrap_or(rest)
    )
}

pub enum QueryUpdate {
    Replace(Vec<(String, String)>),
    Merge(Vec<(String, String)>),
}

impl QueryUpdate {
    fn apply(&self, existing: Option<&str>) -> String {
        match self {
            Self::Replace(params) => {
                let px = params
                    .iter()
                    .map(|(k, v)| (k.as_str(), v.as_str()))
                    .collect();

                querystring::stringify(px)
            },
            Self::Merge(params) => {
                let mut updated = existing
                    .map(querystring::querify)
                    .unwrap_or(Vec::new());

                for (k, v) in params {
                    let member = updated.iter().find(|(ke, _)| ke == k).is_some();

                    if !member {
                        updated.push((k.as_str(), v.as_str()));
                    }
                }

                querystring::stringify(updated)
            }
        }
    }
}

pub struct Builder {
    scheme: Option<String>,
    host: Option<String>,
    port: Option<u16>,
    path: Option<PathUpdate>,
    query: Option<QueryUpdate>,
}

impl Builder {
    pub fn new() -> Self {
        Builder {
            scheme: None,
            host: None,
            port: None,
            path: None,
            query: None
        }
    }

    pub fn scheme(self, scheme: String) -> Self {
        Self {
            scheme: Some(scheme),
            ..self
        }
    }

    pub fn host(self, host: String) -> Self {
        Self {
            host: Some(host),
            ..self
        }
    }

    pub fn port(self, port: u16) -> Self {
        Self {
            port: Some(port),
            ..self
        }
    }

    pub fn path(self, path: PathUpdate) -> Self {
        Self {
            path: Some(path),
            ..self
        }
    }

    pub fn query(self, query: QueryUpdate) -> Self {
        Self {
            query: Some(query),
            ..self
        }
    }

    pub fn build(self) -> Option<Proxy> {
        let scheme = self.scheme?;
        let host = self.host?;

        Some(
            Proxy {
                scheme,
                host,
                port: self.port,
                path: self.path,
                query: self.query
            }
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use http::Request;
    use std::str::FromStr;

    fn mk_req(uri: &str) -> Request<()> {
        http::Request::builder()
            .uri(uri)
            .body(())
            .unwrap()
    }

    #[test]
    fn uri_scheme() {
        let req = mk_req("https://bar.com");
        let scheme_http = http::uri::Scheme::from_str("http").unwrap();
        let host = String::from("foo.com");

        let action = Proxy::builder()
            .scheme(scheme_http.to_string())
            .host(host.clone())
            .build()
            .unwrap();

        let downstream_uri = action
            .transform_req(&req)
            .unwrap()
            .body(())
            .unwrap()
            .uri()
            .clone();

        assert_eq!(
            downstream_uri.scheme().unwrap(),
            &scheme_http,
            "Scheme overrides upstream request scheme"
        );

        assert_eq!(
            downstream_uri.host().unwrap(),
            host.as_str(),
            "Scheme overrides upstream request scheme"
        );
    }

    #[test]
    fn uri_authority() {
        let req = mk_req("https://bar.com");
        let scheme_http = http::uri::Scheme::from_str("http").unwrap();
        let host = String::from("foo.com");

        let action = Proxy::builder()
            .scheme(scheme_http.to_string())
            .host(host.clone())
            .build()
            .unwrap();

        let downstream_uri = action
            .transform_req(&req)
            .unwrap()
            .body(())
            .unwrap()
            .uri()
            .clone();

        assert_eq!(
            downstream_uri.host().unwrap(),
            host.as_str(),
            "Scheme overrides upstream request scheme"
        );
    }

    #[test]
    fn uri_port() {
        let req = mk_req("https://bar.com:8080");
        let scheme_http = http::uri::Scheme::from_str("http").unwrap();
        let host = String::from("foo.com");

        let action_noport = Proxy::builder()
            .scheme(scheme_http.to_string())
            .host(host.clone())
            .build()
            .unwrap();

        let downstream_uri_noport = action_noport
            .transform_req(&req)
            .unwrap()
            .body(())
            .unwrap()
            .uri()
            .clone();

        assert_eq!(
            downstream_uri_noport.port_u16(),
            None,
            "Downstream URI port is not specified when not set in proxy action"
        );

        let action_port = Proxy::builder()
            .scheme(scheme_http.to_string())
            .host(host.clone())
            .port(8081)
            .build()
            .unwrap();

        let downstream_uri_port = action_port
            .transform_req(&req)
            .unwrap()
            .body(())
            .unwrap()
            .uri()
            .clone();

        assert_eq!(
            downstream_uri_port.port_u16(),
            Some(8081),
            "Downstream URI port is specified when set in proxy action"
        );
    }

    #[test]
    fn uri_path() {
        let req = mk_req("https://bar.com:8080/x");
        let scheme_http = http::uri::Scheme::from_str("http").unwrap();
        let host = String::from("foo.com");

        let action_nopath = Proxy::builder()
            .scheme(scheme_http.to_string())
            .host(host.clone())
            .build()
            .unwrap();

        let downstream_uri_nopath = action_nopath
            .transform_req(&req)
            .unwrap()
            .body(())
            .unwrap()
            .uri()
            .clone();

        assert_eq!(
            downstream_uri_nopath.path(),
            "/x",
            "Downstream URI path is carried over from upstream when not set in proxy action"
        );

        let upd_replace = PathUpdate::Replace(String::from("/y"));
        let action_replace = Proxy::builder()
            .scheme(scheme_http.to_string())
            .host(host.clone())
            .path(upd_replace)
            .build()
            .unwrap();

        let downstream_uri_replace = action_replace
            .transform_req(&req)
            .unwrap()
            .body(())
            .unwrap()
            .uri()
            .clone();

        assert_eq!(
            downstream_uri_replace.path(),
            "/y",
            "PathUpdate::Replace replaces path in upstream request"
        );

        let upd_prepend = PathUpdate::Prepend(String::from("/y"));
        let action_prepend = Proxy::builder()
            .scheme(scheme_http.to_string())
            .host(host.clone())
            .path(upd_prepend)
            .build()
            .unwrap();

        let downstream_uri_prepend = action_prepend
            .transform_req(&req)
            .unwrap()
            .body(())
            .unwrap()
            .uri()
            .clone();

        assert_eq!(
            downstream_uri_prepend.path(),
            "/y/x",
            "PathUpdate::Prepend prepends to path in upstream request"
        );

        let upd_append = PathUpdate::Append(String::from("/y"));
        let action_append = Proxy::builder()
            .scheme(scheme_http.to_string())
            .host(host.clone())
            .path(upd_append)
            .build()
            .unwrap();

        let downstream_uri_append = action_append
            .transform_req(&req)
            .unwrap()
            .body(())
            .unwrap()
            .uri()
            .clone();

        assert_eq!(
            downstream_uri_append.path(),
            "/x/y",
            "PathUpdate::Prepend appends to path in upstream request"
        );
    }

    #[test]
    fn uri_query() {
        let req = mk_req("https://bar.com/?x=y");
        let scheme_http = http::uri::Scheme::from_str("http").unwrap();
        let host = String::from("foo.com");

        let action_noquery = Proxy::builder()
            .scheme(scheme_http.to_string())
            .host(host.clone())
            .build()
            .unwrap();

        let downstream_uri_noquery = action_noquery
            .transform_req(&req)
            .unwrap()
            .body(())
            .unwrap()
            .uri()
            .clone();

        assert_eq!(
            downstream_uri_noquery.query().unwrap(),
            "x=y",
            "Downstream URI query is carried over from upstream when not set in proxy action"
        );

        let q_replace = vec![
            (String::from("a"), String::from("b")),
            (String::from("c"), String::from("d")),
        ];
        let upd_replace = QueryUpdate::Replace(q_replace);
        let action_replace = Proxy::builder()
            .scheme(scheme_http.to_string())
            .host(host.clone())
            .query(upd_replace)
            .build()
            .unwrap();

        let downstream_uri_replace = action_replace
            .transform_req(&req)
            .unwrap()
            .body(())
            .unwrap()
            .uri()
            .clone();

        assert_eq!(
            downstream_uri_replace.query().unwrap(),
            "a=b&c=d&",
            "QueryUpdate::Replace replaces query in upstream request"
        );

        let q_merge = vec![
            (String::from("a"), String::from("b")),
            (String::from("c"), String::from("d")),
            (String::from("x"), String::from("z")),
        ];
        let upd_merge = QueryUpdate::Merge(q_merge);
        let action_merge = Proxy::builder()
            .scheme(scheme_http.to_string())
            .host(host.clone())
            .query(upd_merge)
            .build()
            .unwrap();

        let downstream_uri_merge = action_merge
            .transform_req(&req)
            .unwrap()
            .body(())
            .unwrap()
            .uri()
            .clone();

        assert_eq!(
            downstream_uri_merge.query().unwrap(),
            "x=y&a=b&c=d&",
            "QueryUpdate::Replace merges query in upstream request"
        );
    }
}
