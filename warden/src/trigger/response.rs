use http::{Response, StatusCode};

pub struct ResponseTrigger {
    status_code: StatusCodeTrigger,
}

impl ResponseTrigger {
    pub fn applies<T>(&self, rsp: &Response<T>) -> bool {
        self.status_code.applies(rsp)
    }
}

pub enum StatusCodeTrigger {
    Any,
    Exactly(StatusCode),
    OneOf(Vec<StatusCode>),
    NoneOf(Vec<StatusCode>),
}

impl StatusCodeTrigger {
    fn applies<T>(&self, rsp: &Response<T>) -> bool {
        let rsp_status = rsp.status();

        match self {
            Self::Any => true,
            Self::Exactly(code) => rsp_status == *code,
            Self::OneOf(codes) => codes.iter().find(|c| **c == rsp_status).is_some(),
            Self::NoneOf(codes) => codes.iter().find(|c| **c == rsp_status).is_none(),
        }
    }
}

#[cfg(test)]
mod tests {
    mod status_code {
        #[test]
        fn any_applies() {
            let statuses = vec![200, 204, 301, 400, 404, 500];

            for status in statuses {
                let rsp = http::Response::builder().status(status).body(()).unwrap();

                let res = super::super::StatusCodeTrigger::Any.applies(&rsp);
                assert!(res, "StatusCodeTrigger::Any applies to any status code");
            }
        }

        #[test]
        fn exactly_applies() {
            let code_200 = http::StatusCode::from_u16(200).unwrap();
            let code_204 = http::StatusCode::from_u16(204).unwrap();

            let rsp = http::Response::builder().status(code_204).body(()).unwrap();

            let res_eq = super::super::StatusCodeTrigger::Exactly(code_204).applies(&rsp);
            assert!(
                res_eq,
                "StatusCodeTrigger::Exactly applies only to exact status code"
            );

            let res_neq = super::super::StatusCodeTrigger::Exactly(code_200).applies(&rsp);
            assert!(
                !res_neq,
                "StatusCodeTrigger::Exactly applies only to exact status code"
            );
        }

        #[test]
        fn one_of_applies() {
            let code_200 = http::StatusCode::from_u16(200).unwrap();
            let code_204 = http::StatusCode::from_u16(204).unwrap();
            let code_400 = http::StatusCode::from_u16(400).unwrap();

            let rsp = http::Response::builder().status(code_200).body(()).unwrap();

            let res_y =
                super::super::StatusCodeTrigger::OneOf(vec![code_200, code_204]).applies(&rsp);
            assert!(
                res_y,
                "StatusCodeTrigger::OneOf applies only to intersecting status codes"
            );

            let res_n =
                super::super::StatusCodeTrigger::OneOf(vec![code_204, code_400]).applies(&rsp);
            assert!(
                !res_n,
                "StatusCodeTrigger::OneOf applies only to intersecting status codes"
            );
        }

        #[test]
        fn none_of_applies() {
            let code_200 = http::StatusCode::from_u16(200).unwrap();
            let code_204 = http::StatusCode::from_u16(204).unwrap();
            let code_400 = http::StatusCode::from_u16(400).unwrap();

            let rsp = http::Response::builder().status(code_200).body(()).unwrap();

            let res_y =
                super::super::StatusCodeTrigger::NoneOf(vec![code_204, code_400]).applies(&rsp);
            assert!(
                res_y,
                "StatusCodeTrigger::NoneOf applies only to non-intersecting status codes"
            );

            let res_n =
                super::super::StatusCodeTrigger::NoneOf(vec![code_200, code_400]).applies(&rsp);
            assert!(
                !res_n,
                "StatusCodeTrigger::NoneOf applies only to non-intersecting status codes"
            );
        }
    }
}
