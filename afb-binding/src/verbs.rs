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

use crate::prelude::*;

struct NfcAuthCtx {
    mgr: &'static ManagerHandle,
}
AfbVerbRegister!(NfcAuthVerb, nfc_auth_cb, NfcAuthCtx);
fn nfc_auth_cb(rqt: &AfbRequest, _args: &AfbData, ctx: &mut NfcAuthCtx) -> Result<(), AfbError> {

    afb_log_msg!(Debug, rqt,"nfc-authentication request");
    ctx.mgr.nfc_check()?;

    rqt.reply(AFB_NO_DATA, 0);
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

pub(crate) fn register_verbs(api: &mut AfbApi, config: BindingCfg) -> Result<(), AfbError> {

   let auth_event = AfbEvent::new("authorize");
    let auth_mgr = ManagerHandle::new(auth_event, config.nfc_api);

    let auth_nfc = AfbVerb::new("nfc authentication")
        .set_name("nfc-auth")
        .set_callback(Box::new(NfcAuthCtx { mgr: auth_mgr }))
        .set_info("Authenticate with nfc")
        .finalize()?;

    let event = AfbEvent::new("evt");
    let subscribe = AfbVerb::new("subscribe")
        .set_callback(Box::new(SubscribeCtrl { event }))
        .set_info("subscribe Iec6185 event")
        .set_usage("true|false")
        .finalize()?;

    api.add_verb(auth_nfc);
    api.add_verb(subscribe);
    api.add_event(event);
    Ok(())
}
