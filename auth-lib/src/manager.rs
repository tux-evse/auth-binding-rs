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
use std::cell::{RefCell, RefMut};
use typesv4::prelude::*;

pub struct ManagerHandle {
    data_set: RefCell<AuthState>,
    event: &'static AfbEvent,
    scard_api: &'static str,
    ocpp_api: &'static str,
}

impl ManagerHandle {
    pub fn new(
        event: &'static AfbEvent,
        scard_api: &'static str,
        ocpp_api: &'static str,
    ) -> &'static mut Self {
        let handle = ManagerHandle {
            data_set: RefCell::new(AuthState::default()),
            event,
            scard_api,
            ocpp_api,
        };

        // return a static handle to prevent Rust from complaining when moving/sharing it
        Box::leak(Box::new(handle))
    }

    #[track_caller]
    pub fn get_state(&self) -> Result<RefMut<'_, AuthState>, AfbError> {
        match self.data_set.try_borrow_mut() {
            Err(_) => return afb_error!("charging-manager-update", "fail to access &mut data_set"),
            Ok(value) => Ok(value),
        }
    }

    pub fn reset(&self) -> Result<AuthState, AfbError> {
        let mut data_set = self.get_state()?;

        if self.ocpp_api != "" {
            AfbSubCall::call_sync(
                self.event.get_apiv4(),
                self.ocpp_api,
                "transaction",
                OcppTransaction::Stop(0),
            )?;
        }

        data_set.tagid = String::new();
        data_set.auth = AuthMsg::Idle;
        data_set.imax = 0;
        data_set.pmax = 0;
        data_set.ocpp_check=true;
        self.event.push(data_set.auth);
        Ok(data_set.clone())
    }

    pub fn auth_check(&self) -> Result<AuthState, AfbError> {
        let mut data_set = self.get_state()?;
        self.event.push(AuthMsg::Pending);
        let check_tagid = || -> Result<String, AfbError> {
            let response = AfbSubCall::call_sync(
                self.event.get_apiv4(),
                self.scard_api,
                "get-tagid",
                true,
            )?;
            response.get::<String>(0)
        };

        let check_contract = || -> Result<String, AfbError> {
            let response = AfbSubCall::call_sync(
                self.event.get_apiv4(),
                self.scard_api,
                "get-contract",
                true,
            )?;
            response.get::<JsonC>(0)
        };

        let response = match check_tagid() {
            Err(error) => {
                afb_log_msg!(Notice, self.event, "{}", error);
                data_set.tagid = String::new();
                data_set.auth = AuthMsg::Fail;
                return afb_error!("auth-check-fail", "authentication refused");
            }
            Ok(nfc_data) => {
                        data_set.imax = jsonc.default::<u32>("imax", 32)?;
                        data_set.pmax = jsonc.default::<u32>("pmax", 22)?;
                        data_set.ocpp_check = jsonc.default::<bool>("ocpp", true)?;
                 }
                data_set.clone()
        };

        let response = match check_contract() {
            Err(error) => {
                afb_log_msg!(Notice, self.event, "{}", error);
                data_set.tagid = String::new();
                data_set.auth = AuthMsg::Fail;
                return afb_error!("auth-check-fail", "authentication refused");
            }
            Ok(nfc_data) => {
                match JsoncObj::parse(nfc_data.as_str()) {
                    Ok(jsonc) => {
                        data_set.tagid = jsonc.get::<String>("tagid")?;
                        data_set.imax = jsonc.default::<u32>("imax", 32)?;
                        data_set.pmax = jsonc.default::<u32>("pmax", 22)?;
                        data_set.ocpp_check = jsonc.default::<bool>("ocpp", true)?;
                    }
                    Err(_) => {
                        data_set.tagid = nfc_data;
                        data_set.imax = 32;
                        data_set.pmax = 22;
                        data_set.ocpp_check=true;
                    }
                }
                data_set.clone()
            }
        };

        // nfc is ok let check occp tag_id
        if self.ocpp_api != "" && data_set.ocpp_check {
            AfbSubCall::call_sync(
                self.event.get_apiv4(),
                self.ocpp_api,
                "authorize",
                response.tagid.clone(),
            )?;

            // ocpp auth is ok let start ocpp transaction
            AfbSubCall::call_sync(
                self.event.get_apiv4(),
                self.ocpp_api,
                "transaction",
                OcppTransaction::Start(response.tagid.clone()),
            )?;
        }

        self.event.push(response.auth);
        Ok(response)
    }
}
