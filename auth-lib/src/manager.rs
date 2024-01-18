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
}

impl ManagerHandle {
    pub fn new(event: &'static AfbEvent, scard_api: &'static str) -> &'static mut Self {
        let handle = ManagerHandle {
            data_set: RefCell::new(AuthState::default()),
            event,
            scard_api,
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
        let mut data_set= self.get_state()?;
        data_set.tagid= String::new();
        data_set.auth=AuthMsg::Idle;
        data_set.imax=0;
        data_set.pmax=0;
        self.event.push(data_set.auth);
        Ok(data_set.clone())
    }

    pub fn nfc_check(&self) -> Result<AuthState, AfbError> {
        self.event.push(AuthMsg::Pending);
        let check_nfc = || -> Result<String, AfbError> {
            let response = AfbSubCall::call_sync(
                self.event.get_apiv4(),
                self.scard_api,
                "get-contract",
                true,
            )?;
            response.get::<String>(0)
        };

        self.event.push(AuthMsg::Pending);
        let mut data_set= self.get_state()?;
        let response= match check_nfc() {
            Err(error) => {
                data_set.tagid = String::new();
                afb_log_msg!(Notice, self.event,"{}",error);
                self.event.push(AuthMsg::Fail);
                return afb_error!("nfc-check-fail", "authentication refused")
            }
            Ok(value) => {
                data_set.tagid = value;
                self.event.push(AuthMsg::Done);
                data_set.clone()
            }
        };
        Ok(response)
    }
}
