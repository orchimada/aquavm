/*
 * Copyright 2021 Fluence Labs Limited
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

#[test]
// test for github.com/fluencelabs/aquavm/issues/211
// On the versions < 0.20.1 it just crashes
fn issue_211() {
    let peer_1_id = "peer_1_id";

    let script = f!(r#"
    (xor
     (seq
      (seq
       (seq
        (seq
         (null)
         (call %init_peer_id% ("getDataSrv" "idx") [] idx) ; ok=2
        )
        (call %init_peer_id% ("getDataSrv" "nodes") [] nodes) ; ok = [1,2,3]
       )
       (new $nodes2
        (seq
         (seq
          (par
           (fold nodes node
            (par
             (ap node $nodes2)
             (next node)
            )
           )
           (null)
          )
          (call %init_peer_id% ("op" "noop") [$nodes2.$.[idx] nodes]) ; ok="expected result"
         )
         (call %init_peer_id% ("op" "identity") [$nodes2] nodes2-fix) ; ok="expected result"
        )
       )
      )
      (null)
     )
     (call %init_peer_id% ("errorHandlingSrv" "error") [%last_error% 2]) ; ok="error"
    )
    "#);

    let run_params = TestRunParameters::from_init_peer_id(peer_1_id);

    let engine = air_test_framework::TestExecutor::simple(run_params, &script).expect("invalid test executor config");

    let result = engine.execute_one(peer_1_id).unwrap();

    let expected_trace = vec![
        executed_state::scalar_number(2),
        executed_state::scalar(json!([1, 2, 3])),
        executed_state::par(6, 0),
        executed_state::par(1, 4),
        executed_state::ap(Some(0)),
        executed_state::par(1, 2),
        executed_state::ap(Some(0)),
        executed_state::par(1, 0),
        executed_state::ap(Some(0)),
        executed_state::scalar_string("expected result"),
        executed_state::scalar_string("expected result"),
    ];

    let actual_trace = trace_from_result(&result);
    assert_eq!(actual_trace, expected_trace);
}
