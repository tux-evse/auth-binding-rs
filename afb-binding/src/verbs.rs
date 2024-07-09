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

use crate::prelude::*;
use afbv4::prelude::*;
use libauth::prelude::*;
use typesv4::prelude::*;

struct EngyEvtCtx {
    mgr: &'static ManagerHandle,
}
// report value meter to ocpp backend
// AfbEventRegister!(EngyEvtCtrl, engy_event_cb, EngyEvtCtx);
fn engy_event_cb(evt: &AfbEventMsg, args: &AfbRqtData, ctx: &AfbCtxData) -> Result<(), AfbError> {
    let ctx = ctx.get_ref::<EngyEvtCtx>()?;
    let state = args.get::<&EnergyState>(0)?;
    afb_log_msg!(Debug, evt, "energy:{:?}", state.clone());
    ctx.mgr.update_engy_state(state.clone())?;
    Ok(())
}

struct TimerCtx {
    mgr: &'static ManagerHandle,
    evt: &'static AfbEvent,
}
// send charging state every tic ms.
// AfbTimerRegister!(TimerCtrl, timer_callback, TimerCtx);
fn timer_callback(_timer: &AfbTimer, _decount: u32, ctx: &AfbCtxData) -> Result<(), AfbError> {
    let ctx = ctx.get_ref::<TimerCtx>()?;
    let state = ctx.mgr.get_state()?;
    ctx.evt.push(state.clone());
    Ok(())
}

struct LoginRqtCtx {
    mgr: &'static ManagerHandle,
}
// AfbVerbRegister!(LoginRqtVerb, auth_rqt_cb, LoginRqtCtx);
fn auth_rqt_cb(rqt: &AfbRequest, _args: &AfbRqtData, ctx: &AfbCtxData) -> Result<(), AfbError> {
    let ctx = ctx.get_ref::<LoginRqtCtx>()?;
    afb_log_msg!(Debug, rqt, "authentication login request");
    let contract = ctx.mgr.login()?;
    rqt.reply(contract, 0);
    Ok(())
}

struct LogoutRqtCtx {
    mgr: &'static ManagerHandle,
}
// AfbVerbRegister!(LogoutRqtVerb, logout_auth_cb, LogoutRqtCtx);
fn logout_auth_cb(
    rqt: &AfbRequest,
    args: &AfbRqtData,
    ctx: &AfbCtxData,
) -> Result<(), AfbError> {
    let ctx = ctx.get_ref::<LogoutRqtCtx>()?;
    afb_log_msg!(Debug, rqt, "authentication logout request");
    let energy_session= args.get::<i32>(0)?;
    let contract = ctx.mgr.logout(energy_session)?;
    rqt.reply(contract, 0);
    Ok(())
}

struct SubscribeData {
    event: &'static AfbEvent,
}
// AfbVerbRegister!(SubscribeCtrl, subscribe_callback, SubscribeData);
fn subscribe_callback(
    request: &AfbRequest,
    args: &AfbRqtData,
    ctx: &AfbCtxData,
) -> Result<(), AfbError> {
    let ctx = ctx.get_ref::<SubscribeData>()?;
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
// AfbVerbRegister!(StateRequestVerb, state_request_cb, StateRequestCtx);
fn state_request_cb(
    rqt: &AfbRequest,
    args: &AfbRqtData,
    ctx: &AfbCtxData,
) -> Result<(), AfbError> {
    let ctx = ctx.get_ref::<StateRequestCtx>()?;
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

pub(crate) fn register_verbs(api: &mut AfbApi, config: BindingCfg) -> Result<(), AfbError> {
    let event = AfbEvent::new("msg");
    let mgr = ManagerHandle::new(event, config.nfc_api, config.ocpp_api, config.engy_api);

    let engy_handler = AfbEvtHandler::new("energy-evt")
        .set_pattern(to_static_str(format!("{}/*", config.engy_api)))
        .set_callback(engy_event_cb)
        .set_context(EngyEvtCtx { mgr })
        .finalize()?;

    let state_event = AfbEvent::new("state");
    if config.tic > 0 {
        AfbTimer::new("tic-timer")
            .set_period(config.tic)
            .set_decount(0)
            .set_callback(timer_callback)
            .set_context(TimerCtx {
                mgr,
                evt: state_event,
            })
            .start()?;
    }

    let auth_rqt = AfbVerb::new("session authentication")
        .set_name("login")
        .set_callback(auth_rqt_cb)
        .set_context(LoginRqtCtx { mgr })
        .set_info("Login authentication (nfc+ocpp)")
        .finalize()?;

    let auth_reset = AfbVerb::new("reset authentication")
        .set_name("logout")
        .set_callback(logout_auth_cb)
        .set_context(LogoutRqtCtx { mgr })
        .set_info("Logout authenticate")
        .finalize()?;

    let state_verb = AfbVerb::new("auth-state")
        .set_name("state")
        .set_info("session auth-state state")
        .set_actions("['read','subscribe','unsubscribe']")?
        .set_callback(state_request_cb)
        .set_context(StateRequestCtx {
            mgr,
            evt: state_event,
        })
        .finalize()?;

    let subscribe = AfbVerb::new("subscribe")
        .set_callback(subscribe_callback)
        .set_context(SubscribeData { event })
        .set_info("subscribe auth-msg event")
        .set_usage("true|false")
        .finalize()?;

    api.add_evt_handler(engy_handler);
    api.add_verb(auth_rqt);
    api.add_verb(auth_reset);
    api.add_verb(subscribe);
    api.add_verb(state_verb);
    api.add_event(event);
    api.add_event(state_event);
    Ok(())
}
