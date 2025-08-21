# Proxy Canister

This project implements a Proxy Canister that forwards calls from agents or other canisters to target canisters.

It is designed to be integrated with the `icp` CLI tool, enabling users to invoke canister methods that are:
- Restricted to canister-only calls
- Require cycles payments for execution


Please review the [Candid interface](proxy/proxy.did) for details.

## Try with the `canister_info` method of the management canister

The `canister_info` method of the management canister is restricted to canister-only calls.


Make sure that following tools is installed:
- `dfx`: [install guide](https://github.com/dfinity/sdk?tab=readme-ov-file#installing)
- `didc`: `cargo install candid --bin didc`

Deploy the `proxy` on a local network:

```sh
> dfx start --clean --background
> dfx deploy
```

### A successful proxy call

```
# Get a valid canister ID to be used in the following commands
> dfx canister id proxy
uxrrr-q7777-77774-qaaaq-cai

# Encode the call argument as bytes (use the canister ID above)
> didc encode '(record { canister_id = principal "uxrrr-q7777-77774-qaaaq-cai" ; })' -f blob
blob "DIDL\01l\01\b3\c4\b1\f2\04h\01\00\01\0a\ff\ff\ff\ff\ff\90\00\01\01\01"

# Invoke the `proxy` method of the `proxy` canister
> dfx canister call proxy proxy '(
  record {
    canister_id = principal "aaaaa-aa";
    method = "canister_info";
    args = blob "DIDL\01l\01\b3\c4\b1\f2\04h\01\00\01\0a\ff\ff\ff\ff\ff\90\00\01\01\01";
    cycles = 0 : nat;
  },
)'
(
  variant {
    Ok = record {
      result = blob "\44\49\44\4c\14\6c\04\d7\e0\9b\90\02\01\81\cf\ae\f4\0a\02\9f\f4\c1\b6\0b\04\8f\ed\d8\b1\0e\78\6d\68\6e\03\6d\7b\6d\05\6c\04\d6\f6\8e\80\01\78\c0\c3\df\f5\02\78\e6\b3\84\d8\04\06\c2\b9\db\da\0a\0a\6b\02\80\d1\e8\90\02\07\dc\ed\83\b4\0b\08\6c\01\8f\c1\d4\fb\06\68\6c\02\c0\c3\df\f5\02\09\b3\c4\b1\f2\04\68\6e\78\6b\07\9f\90\de\df\02\0b\d7\98\fb\ac\04\0c\bd\81\f4\ce\04\0e\8c\b1\bb\9a\05\0f\c8\9c\b4\c9\0a\11\98\f7\95\de\0a\13\90\c6\90\9a\0f\7f\6c\02\d7\e0\9b\90\02\01\c2\e7\e2\ce\0b\02\6c\02\e3\a6\83\c3\04\0d\81\cf\ae\f4\0a\03\6b\03\c8\bb\8a\70\7f\9c\e9\c6\99\06\7f\9b\aa\eb\ec\08\7f\6c\03\c0\c3\df\f5\02\78\82\bf\f3\a5\0d\78\b6\b8\97\89\0f\03\6c\02\d7\e0\9b\90\02\10\c2\e7\e2\ce\0b\02\6e\01\6c\03\dc\a5\dc\e7\02\12\b3\c4\b1\f2\04\68\8f\ed\d8\b1\0e\78\6c\03\b3\c4\b1\f2\04\68\98\ce\c7\e7\07\78\8f\ed\d8\b1\0e\78\6c\01\d7\e0\9b\90\02\01\01\00\02\01\0a\ff\ff\ff\ff\ff\90\00\00\01\01\01\1d\08\28\9e\85\3f\c3\e9\b3\b0\f3\ed\e3\5f\0c\eb\35\d3\52\f6\ae\33\ab\8c\ea\95\8d\44\02\02\01\20\0e\a1\f4\5a\85\24\6e\89\79\44\27\ff\ff\fb\9f\d8\88\b4\33\11\6a\80\f9\05\da\59\4c\f1\e3\c6\d9\6f\00\03\00\00\00\00\00\00\00";
    }
  },
)
```

### Denied by ingress_message check if sender is not a controller

```
> dfx canister --identity anonymous call proxy proxy '(
  record {
    canister_id = principal "aaaaa-aa";
    method = "canister_info";
    args = blob "DIDL\01l\01\b3\c4\b1\f2\04h\01\00\01\0a\ff\ff\ff\ff\ff\90\00\01\01\01";
    cycles = 0 : nat;
  },
)'
Error: Failed update call.
Caused by: The replica returned a rejection error: reject code CanisterReject, reject message Error from Canister uxrrr-q7777-77774-qaaaq-cai: Canister rejected the message, error code Some("IC0406")
```

### Failure due to insufficient cycles

```
> dfx canister call proxy proxy '(
  record {
    canister_id = principal "aaaaa-aa";
    method = "canister_info";
    args = blob "DIDL\01l\01\b3\c4\b1\f2\04h\01\00\01\0a\ff\ff\ff\ff\ff\90\00\01\01\01";
    cycles = 3_000_000_000_000 : nat;
  },
)'
(
  variant {
    Err = variant {
      InsufficientLiquidCycleBalance = record {
        available = 2_953_114_248_181 : nat;
        required = 3_038_951_520_685 : nat;
      }
    }
  },
)
```
