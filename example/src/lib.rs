use serde::{Serialize, Deserialize};

use taple_sc_rust as sdk;

#[derive(Serialize, Deserialize, Clone)]
struct State {
  pub one: u32,
  pub two: u32,
  pub three: u32
}

#[derive(Serialize, Deserialize)]
enum StateEvent {
  ModOne { data: u32 },
  ModTwo { data: u32 },
  ModThree { data: u32 },
  ModAll { one: u32, two: u32, three: u32 }
}

#[no_mangle]
pub unsafe fn main_function(state_ptr: i32, event_ptr: i32, is_owner: i32) -> u32 {
    sdk::execute_contract(state_ptr, event_ptr, is_owner, contract_logic)
}

fn contract_logic(
  context: &sdk::Context<State, StateEvent>,
  contract_result: &mut sdk::ContractResult<State>,
) {
  let state = &mut contract_result.final_state;
  match context.event {
      StateEvent::ModOne { data } => {
        state.one = data;
      },
      StateEvent::ModTwo { data } => {
        state.two = data;
      },
      StateEvent::ModThree { data } => {
        state.three = data;
      },
      StateEvent::ModAll { one, two, three } => {
        state.one = one;
        state.two = two;
        state.three = three;
      }
  }
  contract_result.success = true;
  contract_result.approval_required = true;
}

#[test]
fn contract_test() {
  let initial_state = State {
    one: 1,
    two: 2,
    three: 3
  };
  let context = sdk::Context {
    initial_state: initial_state.clone(),
    event: StateEvent::ModOne { data: 100 },
    is_owner: false
  };
  let mut result = sdk::ContractResult::new(initial_state);
  contract_logic(&context, &mut result);
  assert_eq!(result.final_state.one, 100);
  assert!(result.success);
  assert!(result.approval_required);
}
