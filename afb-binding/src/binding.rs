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
use typesv4::prelude::*;

pub struct BindingCfg {
    pub nfc_api: &'static str,
    pub ocpp_api: &'static str,
    pub engy_api: &'static str,
    pub tic: u32,
}

// Binding init callback started at binding load time before any API exist
// -----------------------------------------
pub fn binding_init(rootv4: AfbApiV4, jconf: JsoncObj) -> Result<&'static AfbApi, AfbError> {
    afb_log_msg!(Info, rootv4, "config:{}", jconf);

    // add binding custom converter
    auth_registers()?;
    ocpp_registers()?;
    engy_registers()?;

    let uid = jconf.default::<&'static str>("uid", "auth")?;
    let api = jconf.default::<&'static str>("api", uid)?;
    let info = jconf.default::<&'static str>("info", "")?;
    let nfc_api = jconf.default::<&'static str>("nfc_api", "scard")?;
    let ocpp_api = jconf.default::<&'static str>("ocpp_api", "ocpp")?;
    let engy_api = jconf.default::<&'static str>("engy_api", "engy")?;
    let tic = jconf.default::<u32>("tic", 0)?;

    let config = BindingCfg {
        nfc_api,
        ocpp_api,
        engy_api,
        tic,
    };

    // create backend API
    let api = AfbApi::new(api)
        .set_info(info)
        .require_api(nfc_api)
        .require_api(ocpp_api);
    if let Ok(value) = jconf.get::<String>("permission") {
        api.set_permission(AfbPermission::new(to_static_str(value)));
    };

    if let Ok(value) = jconf.get::<i32>("verbosity") {
        api.set_verbosity(value);
    };

    register_verbs(api, config)?;

    Ok(api.finalize()?)
}

// register binding within libafb
AfbBindingRegister!(binding_init);
