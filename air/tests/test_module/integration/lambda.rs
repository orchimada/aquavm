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

use air_test_utils::prelude::*;

use fstrings::f;
use fstrings::format_args_f;

#[test]
fn lambda_not_allowed_for_non_objects_and_arrays() {
    let set_variable_peer_id = "set_variable";
    let mut set_variable_vm = create_avm(echo_call_service(), set_variable_peer_id);

    let local_peer_id = "local_peer_id";
    let mut local_vm = create_avm(echo_call_service(), local_peer_id);

    let script = format!(
        r#"
        (seq
            (call "{0}" ("" "") ["some_string"] string_variable)
            (call "{1}" ("" "") [string_variable.$.some_lambda])
        )
        "#,
        set_variable_peer_id, local_peer_id
    );

    let result = checked_call_vm!(set_variable_vm, "asd", &script, "", "");
    let result = call_vm!(local_vm, "asd", script, "", result.data);

    assert_eq!(result.ret_code, 1003);
}

#[test]
fn lambda_with_string_scalar() {
    let set_variable_peer_id = "set_variable";
    let variables = maplit::hashmap! {
        "string_accessor".to_string() => json!("some_field_name"),
        "value".to_string() => json!({"other_name_1": 0, "some_field_name": 1, "other_name_2": 0})
    };
    let mut set_variable_vm = create_avm(
        set_variables_call_service(variables, VariableOptionSource::FunctionName),
        set_variable_peer_id,
    );

    let local_peer_id = "local_peer_id";
    let mut local_vm = create_avm(echo_call_service(), local_peer_id);

    let script = f!(r#"
        (seq
            (seq
                (call "{set_variable_peer_id}" ("" "string_accessor") [] string_accessor)
                (call "{set_variable_peer_id}" ("" "value") [] value)
            )
            (call "{local_peer_id}" ("" "") [value.$.[string_accessor]])
        )
        "#);

    let result = checked_call_vm!(set_variable_vm, "asd", &script, "", "");
    let result = checked_call_vm!(local_vm, "asd", script, "", result.data);
    let trace = trace_from_result(&result);

    assert_eq!(&trace[2], &executed_state::scalar_number(1u32));
}

#[test]
fn lambda_with_number_scalar() {
    let set_variable_peer_id = "set_variable";
    let variables = maplit::hashmap! {
        "string_accessor".to_string() => json!(1u32),
        "value".to_string() => json!([0, 1, 2])
    };
    let mut set_variable_vm = create_avm(
        set_variables_call_service(variables, VariableOptionSource::FunctionName),
        set_variable_peer_id,
    );

    let local_peer_id = "local_peer_id";
    let mut local_vm = create_avm(echo_call_service(), local_peer_id);

    let script = f!(r#"
        (seq
            (seq
                (call "{set_variable_peer_id}" ("" "string_accessor") [] string_accessor)
                (call "{set_variable_peer_id}" ("" "value") [] value)
            )
            (call "{local_peer_id}" ("" "") [value.$.[string_accessor]])
        )
        "#);

    let result = checked_call_vm!(set_variable_vm, "asd", &script, "", "");
    let result = checked_call_vm!(local_vm, "asd", script, "", result.data);
    let trace = trace_from_result(&result);

    assert_eq!(&trace[2], &executed_state::scalar_number(1u32));
}

#[test]
fn lambda_with_scalar_join() {
    let set_variable_peer_id = "set_variable";
    let variables = maplit::hashmap! {
        "string_accessor".to_string() => json!("some_field_name"),
        "value".to_string() => json!({"other_name_1": 0, "some_field_name": 1, "other_name_2": 0})
    };
    let mut set_variable_vm = create_avm(
        set_variables_call_service(variables, VariableOptionSource::FunctionName),
        set_variable_peer_id,
    );

    let local_peer_id = "local_peer_id";
    let mut local_vm = create_avm(echo_call_service(), local_peer_id);

    let script = f!(r#"
        (seq
            (par
                (call "non_exist_peer_id" ("" "string_accessor") [] string_accessor)
                (call "{set_variable_peer_id}" ("" "value") [] value)
            )
            (call "{local_peer_id}" ("" "") [value.$.[string_accessor]])
        )
        "#);

    let result = checked_call_vm!(set_variable_vm, "asd", &script, "", "");
    let result = checked_call_vm!(local_vm, "asd", script, "", result.data);
    let trace = trace_from_result(&result);

    assert_eq!(&trace[3], &executed_state::request_sent_by("set_variable"));
}