// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use async_trait::async_trait;
use serde_json::{Value, json};
use wiremock::{
    Mock, Request, Respond, ResponseTemplate,
    matchers::{body_partial_json, method},
};

// for now we don't worry about errors, proper proto encoding of dynamic content, etc.
struct RpcRequestResponder {
    response_value: String,
}

impl RpcRequestResponder {
    fn new(value: impl Into<String>) -> RpcRequestResponder {
        RpcRequestResponder {
            response_value: value.into(),
        }
    }
}

#[async_trait]
impl Respond for RpcRequestResponder {
    fn respond(&self, request: &Request) -> ResponseTemplate {
        // Convert raw body bytes to string
        let body_str = String::from_utf8_lossy(&request.body).to_string();

        // Try to parse as JSON to extract request id
        let Ok(json) = serde_json::from_str::<Value>(&body_str) else {
            return ResponseTemplate::new(400).set_body_string("bad request");
        };
        let Some(id) = json.get("id").and_then(|v| v.as_str()) else {
            return ResponseTemplate::new(400).set_body_string("bad request");
        };

        let response = json!({
           "jsonrpc": "2.0",
           "id": id,
           "result": {
             "response": {
               "code": 0,
               "log": "",
               "info": "",
               "index": "0",
               "key": null,
               "value": self.response_value,
               "proofOps": null,
               "height": "20370499",
               "codespace": ""
             }
           }
        });

        ResponseTemplate::new(200).set_body_json(response)
    }
}

pub fn rpc_mock(path: impl Into<String>, response: impl Respond + 'static) -> Mock {
    // sample abci_query:
    // {
    //   "jsonrpc": "2.0",
    //   "id": "78d0441c-e944-4bde-bb14-bd80a039b849",
    //   "method": "abci_query",
    //   "params": {
    //     "path": "/cosmos.auth.v1beta1.Query/Account",
    //     "data": "0A286E31767A6B3834776361366A673273793263336572617063706B7A7839706E667675343536637264",
    //     "prove": false
    //   }
    // }
    //
    // and sample response:
    // {
    //   "jsonrpc": "2.0",
    //   "id": "7457fb86-5afe-467c-b6e4-e80f1b9dcf49",
    //   "result": {
    //     "response": {
    //       "code": 0,
    //       "log": "",
    //       "info": "",
    //       "index": "0",
    //       "key": null,
    //       "value": "CpsBCiAvY29zbW9zLmF1dGgudjFiZXRhMS5CYXNlQWNjb3VudBJ3CihuMTgwbjl4Zmpqdm15bHRnamhuY2o5aHU1Y3g3eWwwdXg2ZDZ5bDVoEkYKHy9jb3Ntb3MuY3J5cHRvLnNlY3AyNTZrMS5QdWJLZXkSIwohAy5JwNe7XxGCjqtxcfTHl/OM4c9yBraP+ve9O2WXZlN7GMN7IAU=",
    //       "proofOps": null,
    //       "height": "20370499",
    //       "codespace": ""
    //     }
    //   }
    // }

    let path_match = body_partial_json(json!({
        "params": {
            "path": path.into(),
        }
    }));
    let rpc_match = body_partial_json(json!({
        "jsonrpc": "2.0",
        "method": "abci_query"
    }));
    Mock::given(method("POST"))
        .and(rpc_match)
        .and(path_match)
        .respond_with(response)
}

pub fn get_account_exists() -> Mock {
    // technically this returns account information for another account, but we never make such consistency check
    // : )
    rpc_mock(
        "/cosmos.auth.v1beta1.Query/Account",
        RpcRequestResponder::new(
            "CpsBCiAvY29zbW9zLmF1dGgudjFiZXRhMS5CYXNlQWNjb3VudBJ3CihuMTgwbjl4Zmpqdm15bHRnamhuY2o5aHU1Y3g3eWwwdXg2ZDZ5bDVoEkYKHy9jb3Ntb3MuY3J5cHRvLnNlY3AyNTZrMS5QdWJLZXkSIwohAy5JwNe7XxGCjqtxcfTHl/OM4c9yBraP+ve9O2WXZlN7GMN7IAU=",
        ),
    )
}
