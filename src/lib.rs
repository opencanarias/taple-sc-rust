mod error;
mod externf;
mod value_wrapper;
use borsh::{BorshDeserialize, BorshSerialize};
use error::Error;
use json_patch::{patch, Patch};
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub use self::value_wrapper::ValueWrapper;

#[derive(Serialize, Deserialize, Debug)]
pub struct Context<State, Event> {
    pub initial_state: State,
    pub event: Event,
    pub is_owner: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ContractResult<State> {
    pub final_state: State,
    pub approval_required: bool,
    pub success: bool,
}

#[derive(BorshSerialize)]
struct ContractResultBorsh {
    pub final_state: ValueWrapper,
    pub approval_required: bool,
    pub success: bool,
}

impl ContractResultBorsh {
    pub fn error() -> Self {
        Self {
            final_state: ValueWrapper(serde_json::Value::Null),
            approval_required: false,
            success: false,
        }
    }
}

impl<State> ContractResult<State> {
    pub fn new(state: State) -> Self {
        Self {
            final_state: state,
            approval_required: false,
            success: false,
        }
    }
}

pub fn execute_contract<F, State, Event>(
    state_ptr: i32,
    event_ptr: i32,
    is_owner: i32,
    callback: F,
) -> u32
where
    State: for<'a> Deserialize<'a> + Serialize + Clone,
    Event: for<'a> Deserialize<'a> + Serialize,
    F: Fn(&Context<State, Event>, &mut ContractResult<State>),
{
    {
        'process: {
            let Ok(state_value) = deserialize(get_from_context(state_ptr)) else {
                break 'process;
            };
            let Ok(state) = serde_json::from_value::<State>(state_value.0) else {
                break 'process;
            };
            let Ok(event_value) = deserialize(get_from_context(event_ptr)) else {
                break 'process;
            };
            let Ok(event) = serde_json::from_value::<Event>(event_value.0) else {
                break 'process;
            };
            let is_owner = if is_owner == 1 { true } else { false };
            let context = Context {
                initial_state: state.clone(),
                event,
                is_owner,
            };
            let mut contract_result = ContractResult::new(state);
            callback(&context, &mut contract_result);
            let Ok(state_value) = serde_json::to_value(&contract_result.final_state) else {
                break 'process;
            };
            let result = ContractResultBorsh {
                final_state: ValueWrapper(state_value),
                approval_required: contract_result.approval_required,
                success: contract_result.success,
            };
            // Después de haber sido modificado debemos guardar el nuevo estado.
            // Sería interesante no tener que guardar estado si el evento no es modificante
            let Ok(result_ptr) = store(&result) else {
              break 'process;
            };
            return result_ptr;
        };
        let result = ContractResultBorsh::error();
        store(&result).expect("Contract store process failed")
    }
}

fn deserialize(bytes: Vec<u8>) -> Result<ValueWrapper, Error> {
    BorshDeserialize::try_from_slice(&bytes).map_err(|_| Error::DeserializationError)
}

fn serialize<S: BorshSerialize>(data: S) -> Result<Vec<u8>, Error> {
    data.try_to_vec().map_err(|_| Error::SerializationError)
}

fn get_from_context(pointer: i32) -> Vec<u8> {
    let data = unsafe {
        let len = externf::pointer_len(pointer);
        let mut data = vec![];
        for i in 0..len {
            data.push(externf::read_byte(pointer + i));
        }
        data
    };
    data
}

pub fn apply_patch<State: for<'a> Deserialize<'a> + Serialize>(
    patch_arg: Value,
    state: &State,
) -> Result<State, i32> {
    let patch_data: Patch = serde_json::from_value(patch_arg).unwrap();
    let mut state = serde_json::to_value(state).unwrap();
    patch(&mut state, &patch_data).unwrap();
    Ok(serde_json::from_value(state).unwrap())
}

fn store(data: &ContractResultBorsh) -> Result<u32, Error> {
    let bytes = serialize(&data).map_err(|_| Error::SerializationError)?;
    unsafe {
        let ptr = externf::alloc(bytes.len() as u32) as u32;
        for (index, byte) in bytes.into_iter().enumerate() {
            externf::write_byte(ptr, index as u32, byte);
        }
        Ok(ptr)
    }
}
