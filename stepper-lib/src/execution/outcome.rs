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

use crate::AquamarineError;

use stepper_interface::StepperOutcome;
use stepper_interface::STEPPER_SUCCESS;

use serde::Serialize;
use std::hash::Hash;

/// Create StepperOutcome from supplied data and next_peer_pks,
/// set ret_code to STEPPER_SUCCESS.
pub(crate) fn success<T>(data: &T, next_peer_pks: Vec<String>) -> StepperOutcome
where
    T: ?Sized + Serialize,
{
    let data = serde_json::to_vec(data).expect("default serializer shouldn't fail");
    let next_peer_pks = dedup(next_peer_pks);

    StepperOutcome {
        ret_code: STEPPER_SUCCESS,
        error_message: String::new(),
        data,
        next_peer_pks,
    }
}

/// Create StepperOutcome from supplied data and error,
/// set ret_code based on the error.
pub(crate) fn error_from_raw_data(data: impl Into<Vec<u8>>, err: AquamarineError) -> StepperOutcome {
    let ret_code = err.to_error_code();
    let data = data.into();

    StepperOutcome {
        ret_code,
        error_message: format!("{}", err),
        data,
        next_peer_pks: vec![],
    }
}

/// Create StepperOutcome from supplied data, next_peer_pks and error,
/// set ret_code based on the error.
pub(crate) fn error_from_data<T>(data: &T, next_peer_pks: Vec<String>, err: AquamarineError) -> StepperOutcome
where
    T: ?Sized + Serialize,
{
    let ret_code = err.to_error_code();
    let data = serde_json::to_vec(data).expect("default serializer shouldn't fail");
    let next_peer_pks = dedup(next_peer_pks);

    StepperOutcome {
        ret_code,
        error_message: format!("{}", err),
        data,
        next_peer_pks,
    }
}

/// Deduplicate values in a supplied vector.
fn dedup<T: Eq + Hash>(mut vec: Vec<T>) -> Vec<T> {
    use std::collections::HashSet;

    let set: HashSet<_> = vec.drain(..).collect();
    set.into_iter().collect()
}