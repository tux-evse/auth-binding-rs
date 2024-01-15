/*
 * Copyright (C) 2015-2022 IoT.bzh Company
 * Author: Fulup Ar Foll <fulup@iot.bzh>
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *   http://www.apache.org/licenses/LICENSE-2.0
 *
 */

use afbv4::prelude::*;
use libauth::prelude::*;
use typesv4::prelude::*;
use crate::prelude::*;

struct NfcAuthCtx {
    mgr: &'static ManagerHandle,
}
AfbVerbRegister!(NfcAuthVerb, nfc_auth_cb, NfcAuthCtx);
fn nfc_auth_cb(rqt: &AfbRequest, _args: &AfbData, ctx: &mut NfcAuthCtx) -> Result<(), AfbError> {
    afb_log_msg!(Debug, rqt, "nfc-authentication request");
    let contract= ctx.mgr.nfc_check()?;
    rqt.reply(contract, 0);
    Ok(())
}

struct SubscribeData {
    event: &'static AfbEvent,
}
AfbVerbRegister!(SubscribeCtrl, subscribe_callback, SubscribeData);
fn subscribe_callback(
    request: &AfbRequest,
    args: &AfbData,
    ctx: &mut SubscribeData,
) -> Result<(), AfbError> {
    let subcription = args.get::<bool>(0)?;
    if subcription {
        ctx.event.subscribe(request)?;
    } else {
        ctx.event.unsubscribe(request)?;
    }
    request.reply(AFB_NO_DATA, 0);
    Ok(())
}

struct StateRequestCtx {
    mgr: &'static ManagerHandle,
    evt: &'static AfbEvent,
}
AfbVerbRegister!(StateRequestVerb, state_request_cb, StateRequestCtx);
fn state_request_cb(
    rqt: &AfbRequest,
    args: &AfbData,
    ctx: &mut StateRequestCtx,
) -> Result<(), AfbError> {

    match args.get::<&AuthAction>(0)? {
        AuthAction::READ => {
            let data_set = ctx.mgr.get_state()?;
            rqt.reply(data_set.clone(), 0);
        }

        AuthAction::SUBSCRIBE => {
            afb_log_msg!(Notice, rqt, "Subscribe {}", ctx.evt.get_uid());
            ctx.evt.subscribe(rqt)?;
            rqt.reply(AFB_NO_DATA, 0);
        }

        AuthAction::UNSUBSCRIBE => {
            afb_log_msg!(Notice, rqt, "Unsubscribe {}", ctx.evt.get_uid());
            ctx.evt.unsubscribe(rqt)?;
            rqt.reply(AFB_NO_DATA, 0);
        }
    }
    Ok(())
}

struct TimerCtx {
    mgr: &'static ManagerHandle,
    evt: &'static AfbEvent,
}
// send charging state every tic ms.
AfbTimerRegister!(TimerCtrl, timer_callback, TimerCtx);
fn timer_callback(_timer: &AfbTimer, _decount: u32, ctx: &mut TimerCtx) -> Result<(), AfbError> {
    let state= ctx.mgr.get_state()?;
    ctx.evt.push(state.clone());
    Ok(())
}

pub(crate) fn register_verbs(api: &mut AfbApi, config: BindingCfg) -> Result<(), AfbError> {
    let event = AfbEvent::new("evt");
    let mgr = ManagerHandle::new(event, config.nfc_api);

    let state_event = AfbEvent::new("state");
    AfbTimer::new("tic-timer")
        .set_period(config.tic)
        .set_decount(0)
        .set_callback(Box::new(TimerCtx {
           mgr,
           evt: state_event,
        }))
        .start()?;

    let auth_nfc = AfbVerb::new("nfc authentication")
        .set_name("nfc-auth")
        .set_callback(Box::new(NfcAuthCtx { mgr }))
        .set_info("Authenticate with nfc")
        .finalize()?;

    let state_verb = AfbVerb::new("auth-state")
        .set_name("state")
        .set_info("session authentication state")
        .set_action("['read','subscribe','unsubscribe']")?
        .set_callback(Box::new(StateRequestCtx {
            mgr,
            evt: state_event,
        }))
        .finalize()?;

    let subscribe = AfbVerb::new("subscribe")
        .set_callback(Box::new(SubscribeCtrl { event }))
        .set_info("subscribe authentication event")
        .set_usage("true|false")
        .finalize()?;

    api.add_verb(auth_nfc);
    api.add_verb(subscribe);
    api.add_verb(state_verb);
    api.add_event(event);
    api.add_event(state_event);
    Ok(())
}
