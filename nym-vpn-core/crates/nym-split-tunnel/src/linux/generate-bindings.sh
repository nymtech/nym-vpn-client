#!/usr/bin/env bash

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
cd "$SCRIPT_DIR"

bindgen "include/bindings.h" -o ./bindings.rs \
    --raw-line "#![allow(non_camel_case_types)]" \
    --raw-line "#![allow(dead_code)]" \
    --raw-line "#![allow(unsafe_op_in_unsafe_fn)]" \
    --raw-line "use libc::{nlmsghdr, proc_cn_event, proc_cn_mcast_op};" \
    --raw-line "pub type exec_proc_event = proc_event__bindgen_ty_1_exec_proc_event;" \
    --raw-line "pub type fork_proc_event = proc_event__bindgen_ty_1_fork_proc_event;" \
    --raw-line "pub type exit_proc_event = proc_event__bindgen_ty_1_exit_proc_event;" \
    --no-derive-debug \
    --blocklist-item "nlmsghdr" \
    --blocklist-item "proc_cn_event" \
    --blocklist-item "proc_cn_mcast_op" \
    --allowlist-item "nlcn_subscribe_msg" \
    --allowlist-item "nlcn_subscribe_payload" \
    --allowlist-item "nlcn_event_msg" \
    --allowlist-item "nlcn_event_payload"
