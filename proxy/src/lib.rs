use candid::{CandidType, Principal};
use ic_cdk::api::{is_controller, msg_caller};
use ic_cdk::call::{Call, CallFailed};
use serde::{Deserialize, Serialize};

// nat in candid can be represented as u128 in Rust
type Cycles = u128;

#[derive(CandidType, Serialize, Deserialize, Debug, Clone)]
struct ProxyArgs {
    canister_id: Principal,
    method: String,
    #[serde(with = "serde_bytes")]
    args: Vec<u8>,
    cycles: Cycles,
}

type ProxyResult = Result<ProxySucceed, ProxyError>;

#[derive(CandidType, Serialize, Deserialize, Debug, Clone)]
struct ProxySucceed {
    #[serde(with = "serde_bytes")]
    result: Vec<u8>,
}

#[derive(CandidType, Serialize, Deserialize, Debug, Clone)]
enum ProxyError {
    InsufficientCycles { available: Cycles, required: Cycles },
    CallFailed { reason: String },
    UnauthorizedUser,
}

/// The proxy service is only accessible to the controllers.
///
/// The inspect_message hook rejects unauthorized ingress messages in non-replicated execution,
/// preventing unnecessary cycle consumption.
#[ic_cdk::inspect_message]
fn inspect_message() {
    let msg_caller = msg_caller();
    if is_controller(&msg_caller) {
        ic_cdk::api::accept_message();
    }
}

#[ic_cdk::update]
async fn proxy(args: ProxyArgs) -> ProxyResult {
    // Though the inspect_message hook can deny unauthorized ingress messages,
    // we still need to check the caller's identity here.
    // Because inter-canister calls are not subject to the inspect_message check.
    let msg_caller = msg_caller();
    if !is_controller(&msg_caller) {
        return Err(ProxyError::UnauthorizedUser);
    }

    let res = Call::bounded_wait(args.canister_id, &args.method)
        .with_raw_args(&args.args)
        .with_cycles(args.cycles)
        .await;
    match res {
        Ok(response) => {
            let result = response.into_bytes();
            Ok(ProxySucceed { result })
        }
        Err(call_failed) => match call_failed {
            CallFailed::InsufficientLiquidCycleBalance(e) => {
                Err(ProxyError::InsufficientCycles {
                    available: e.available,
                    required: e.required,
                })
            }
            CallFailed::CallPerformFailed(_) => Err(ProxyError::CallFailed {
                reason: "call_perform failed synchronously".to_string(),
            }),
            CallFailed::CallRejected(e) => {
                let reject_code = match e.reject_code() {
                    Ok(code_name) => code_name.to_string(),
                    Err(_) => "UnrecognizedRejectCode".to_string(),
                };
                let raw_reject_code = e.raw_reject_code();
                let reject_message = e.reject_message().to_owned();
                let reason = format!(
                    "Call Rejected, reject_code: {} ({}), reject_message: {}",
                    reject_code, raw_reject_code, reject_message
                );
                Err(ProxyError::CallFailed { reason })
            }
        },
    }
}
