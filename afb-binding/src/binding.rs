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

pub(crate) fn to_static_str(value: String) -> &'static str {
    Box::leak(value.into_boxed_str())
}

pub struct BindingCfg {
    pub nfc_api: &'static str,
}

// Binding init callback started at binding load time before any API exist
// -----------------------------------------
pub fn binding_init(rootv4: AfbApiV4, jconf: JsoncObj) -> Result<&'static AfbApi, AfbError> {
    afb_log_msg!(Info, rootv4, "config:{}", jconf);

    // add binding custom converter
    auth_register()?;

    let uid = if let Ok(value) = jconf.get::<String>("uid") {
        to_static_str(value)
    } else {
        "auth"
    };

    let api = if let Ok(value) = jconf.get::<String>("api") {
        to_static_str(value)
    } else {
        uid
    };

    let info = if let Ok(value) = jconf.get::<String>("info") {
        to_static_str(value)
    } else {
        ""
    };

    let nfc_api = to_static_str(jconf.get::<String>("nfc_api")?);

    let config = BindingCfg {
        nfc_api
    };

    // create backend API
    let api = AfbApi::new(api).set_info(info);
    if let Ok(value) = jconf.get::<String>("permission") {
        api.set_permission(AfbPermission::new(to_static_str(value)));
    };

    register_verbs(api, config)?;

    Ok(api.finalize()?)
}

// register binding within libafb
AfbBindingRegister!(binding_init);
