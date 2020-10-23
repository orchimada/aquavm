/*
 * Copyright 2020 Fluence Labs Limited
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

use aqua_test_utils::create_aqua_vm;
use aqua_test_utils::unit_call_service;
use aquamarine_vm::vec1::Vec1;
use aquamarine_vm::HostExportedFunc;
use aquamarine_vm::IValue;

use pretty_assertions::assert_eq;
use serde_json::json;

type JValue = serde_json::Value;

#[test]
fn join_chat() {
    use std::collections::HashSet;

    let members_call_service1: HostExportedFunc = Box::new(|_, _| -> Option<IValue> {
        Some(IValue::Record(
            Vec1::new(vec![
                IValue::S32(0),
                IValue::String(String::from(r#"[["A", "Relay1"], ["B", "Relay2"]]"#)),
            ])
            .unwrap(),
        ))
    });

    let mut relay_1 = create_aqua_vm(unit_call_service(), "Relay1");
    let mut relay_2 = create_aqua_vm(unit_call_service(), "Relay2");
    let mut remote = create_aqua_vm(members_call_service1, "Remote");
    let mut client_1 = create_aqua_vm(unit_call_service(), "A");
    let mut client_2 = create_aqua_vm(unit_call_service(), "B");

    let script = String::from(
        r#"
            (seq (
                (call ("Relay1" ("identity" "") () void1[]))
                (seq (
                    (call ("Remote" ("552196ea-b9b2-4761-98d4-8e7dba77fac4" "add") () void2[]))
                    (seq (
                        (call ("Remote" ("920e3ba3-cbdf-4ae3-8972-0fa2f31fffd9" "get_users") () members))
                        (fold (members m
                            (par (
                                (seq (
                                    (call (m.$.[1] ("identity" "") () void[]))
                                    (call (m.$.[0] ("fgemb3" "add") () void3[]))
                                ))
                                (next m)
                            ))
                        ))
                    ))
                ))
            ))
        "#,
    );

    let client_1_res = client_1
        .call(json!(["asd", script, "{}", "{}"]))
        .expect("should be successful");

    let client_1_res_json: JValue = serde_json::from_str(&client_1_res.data).expect("stepper should return valid json");

    let client_1_right_json = json!( {
        "__call": [
            { "call": "request_sent" },
        ]
    });

    assert_eq!(client_1_res_json, client_1_right_json);
    assert_eq!(client_1_res.next_peer_pks, vec![String::from("Relay1")]);

    let relay_1_res = relay_1
        .call(json!(["asd", script, client_1_res.data, "{}"]))
        .expect("should be successful");

    let relay_1_res_json: JValue = serde_json::from_str(&relay_1_res.data).expect("stepper should return valid json");

    let relay_1_right_json = json!( {
        "void1": ["test"],
        "__call": [
            { "call": "executed" },
            { "call": "request_sent" },
        ]
    });

    assert_eq!(relay_1_res_json, relay_1_right_json);
    assert_eq!(relay_1_res.next_peer_pks, vec![String::from("Remote")]);

    let remote_res = remote
        .call(json!(["asd", script, relay_1_res.data, "{}"]))
        .expect("should be successful");

    let remote_res_json: JValue = serde_json::from_str(&remote_res.data).expect("stepper should return valid json");

    let remote_right_json = json!( {
        "void1": ["test"],
        "void2": [[["A", "Relay1"], ["B", "Relay2"]]],
        "members": [["A", "Relay1"], ["B", "Relay2"]],
        "__call": [
            { "call": "executed" },
            { "call": "executed" },
            { "call": "executed" },
            { "par": [1, 2] },
            { "call": "request_sent" },
            { "par": [1, 0] },
            { "call": "request_sent" },
        ]
    });

    let remote_res_next_peer_pks: HashSet<_> = remote_res.next_peer_pks.iter().map(|s| s.as_str()).collect();
    let next_peer_pks_right = maplit::hashset! {
        "Relay1",
        "Relay2",
    };

    assert_eq!(remote_res_json, remote_right_json);
    assert_eq!(remote_res_next_peer_pks, next_peer_pks_right);

    let relay_1_res = relay_1
        .call(json!(["asd", script, remote_res.data, "{}"]))
        .expect("should be successful");

    let relay_1_res_json: JValue = serde_json::from_str(&relay_1_res.data).expect("stepper should return valid json");

    let relay_1_right_json = json!( {
        "void1": ["test"],
        "void2": [[["A", "Relay1"], ["B", "Relay2"]]],
        "void": ["test"],
        "members": [["A", "Relay1"], ["B", "Relay2"]],
        "__call": [
            { "call": "executed" },
            { "call": "executed" },
            { "call": "executed" },
            { "par": [2, 2] },
            { "call": "executed" },
            { "call": "request_sent" },
            { "par": [1, 0] },
            { "call": "request_sent" },
        ]
    });

    assert_eq!(relay_1_res_json, relay_1_right_json);
    assert_eq!(relay_1_res.next_peer_pks, vec![String::from("A")]);

    let client_1_res = client_1
        .call(json!(["asd", script, relay_1_res.data, "{}"]))
        .expect("should be successful");

    let client_1_res_json: JValue = serde_json::from_str(&client_1_res.data).expect("stepper should return valid json");

    let client_1_right_json = json!( {
        "void1": ["test"],
        "void2": [[["A", "Relay1"], ["B", "Relay2"]]],
        "void": ["test"],
        "void3": ["test"],
        "members": [["A", "Relay1"], ["B", "Relay2"]],
        "__call": [
            { "call": "executed" },
            { "call": "executed" },
            { "call": "executed" },
            { "par": [2, 2] },
            { "call": "executed" },
            { "call": "executed" },
            { "par": [1, 0] },
            { "call": "request_sent" },
        ]
    });

    assert_eq!(client_1_res_json, client_1_right_json);
    assert_eq!(client_1_res.next_peer_pks, Vec::<String>::new());

    let relay_2_res = relay_2
        .call(json!(["asd", script, remote_res.data, "{}"]))
        .expect("should be successful");

    let relay_2_res_json: JValue = serde_json::from_str(&relay_2_res.data).expect("stepper should return valid json");

    let relay_2_right_json = json!( {
        "void1": ["test"],
        "void2": [[["A", "Relay1"], ["B", "Relay2"]]],
        "void": ["test"],
        "members": [["A", "Relay1"], ["B", "Relay2"]],
        "__call": [
            { "call": "executed" },
            { "call": "executed" },
            { "call": "executed" },
            { "par": [1, 3] },
            { "call": "request_sent" },
            { "par": [2, 0] },
            { "call": "executed" },
            { "call": "request_sent" },
        ]
    });

    assert_eq!(relay_2_res_json, relay_2_right_json);
    assert_eq!(relay_2_res.next_peer_pks, vec![String::from("B")]);

    let client_2_res = client_2
        .call(json!(["asd", script, relay_2_res.data, "{}"]))
        .expect("should be successful");

    let client_2_res_json: JValue = serde_json::from_str(&client_2_res.data).expect("stepper should return valid json");

    let client_2_right_json = json!( {
        "void1": ["test"],
        "void2": [[["A", "Relay1"], ["B", "Relay2"]]],
        "void": ["test"],
        "void3": ["test"],
        "members": [["A", "Relay1"], ["B", "Relay2"]],
        "__call": [
            { "call": "executed" },
            { "call": "executed" },
            { "call": "executed" },
            { "par": [1, 3] },
            { "call": "request_sent" },
            { "par": [2, 0] },
            { "call": "executed" },
            { "call": "executed" },
        ]
    });

    assert_eq!(client_2_res_json, client_2_right_json);
    assert_eq!(client_2_res.next_peer_pks, Vec::<String>::new());
}

#[test]
fn join() {
    env_logger::init();

    let members_call_service1: HostExportedFunc = Box::new(|_, _| -> Option<IValue> {
        Some(IValue::Record(
            Vec1::new(vec![IValue::S32(0), IValue::String(String::from(r#"[["A"], ["B"]]"#))]).unwrap(),
        ))
    });

    let mut relay_1 = create_aqua_vm(unit_call_service(), "Relay1");
    let mut remote = create_aqua_vm(members_call_service1, "Remote");
    let mut client_1 = create_aqua_vm(unit_call_service(), "A");

    let script = String::from(
        r#"
            (seq (
                (call ("Relay1" ("identity" "") () void1[]))
                (seq (
                        (call ("Remote" ("920e3ba3-cbdf-4ae3-8972-0fa2f31fffd9" "get_users") () members))
                        (fold (members m
                            (par (
                                (seq (
                                    (call ("Relay1" ("identity" "") () void[]))
                                    (call ("A" ("fgemb3" "add") (m) void3[]))
                                ))
                                (next m)
                            ))
                        ))
                ))
            ))
        "#,
    );

    let client_1_res = client_1
        .call(json!(["asd", script, "{}", "{}"]))
        .expect("should be successful");

    let relay_1_res = relay_1
        .call(json!(["asd", script, client_1_res.data, "{}"]))
        .expect("should be successful");

    let remote_res = remote
        .call(json!(["asd", script, relay_1_res.data, "{}"]))
        .expect("should be successful");

    let relay_1_res = relay_1
        .call(json!(["asd", script, remote_res.data, "{}"]))
        .expect("should be successful");

    let client_1_res = client_1
        .call(json!(["asd", script, relay_1_res.data, "{}"]))
        .expect("should be successful");

    let client_1_res_json: JValue = serde_json::from_str(&client_1_res.data).expect("stepper should return valid json");

    let client_1_right_json = json!( {
        "void1": ["test"],
        "void": ["test", "test"],
        "void3": ["test", "test"],
        "members": [["A"], ["B"]],
        "__call": [
            { "call": "executed" },
            { "call": "executed" },
            { "par": [2, 3] },
            { "call": "executed" },
            { "call": "executed" },
            { "par": [2, 0] },
            { "call": "executed" },
            { "call": "executed" },
        ]
    });

    assert_eq!(client_1_res_json, client_1_right_json);
    assert_eq!(client_1_res.next_peer_pks, Vec::<String>::new());
}
