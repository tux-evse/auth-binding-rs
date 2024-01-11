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

pub struct ManagerState {
    contact: Option<String>,
}

impl ManagerState {
    pub fn default() -> Self {
        // Warning: unit are value*100
        ManagerState { contact: None }
    }
}

pub struct ManagerHandle {
    data_set: RefCell<ManagerState>,
    event: &'static AfbEvent,
    scard_api: &'static str,
}

impl ManagerHandle {
    pub fn new(event: &'static AfbEvent, scard_api: &'static str) -> &'static mut Self {
        let handle = ManagerHandle {
            data_set: RefCell::new(ManagerState::default()),
            event,
            scard_api,
        };

        // return a static handle to prevent Rust from complaining when moving/sharing it
        Box::leak(Box::new(handle))
    }

    #[track_caller]
    fn get_state(&self) -> Result<RefMut<'_, ManagerState>, AfbError> {
        match self.data_set.try_borrow_mut() {
            Err(_) => return afb_error!("charging-manager-update", "fail to access &mut data_set"),
            Ok(value) => Ok(value),
        }
    }

    pub fn nfc_check(&self) -> Result<&Self, AfbError> {
        let mut data_set = self.get_state()?;

        let check_nfc = || -> Result<String, AfbError> {
            let response= AfbSubCall::call_sync(self.event.get_apiv4(), self.scard_api, "get-contract", true)?;
            response.get::<String>(0)
        };

        self.event.push(AuthState::Pending);
        data_set.contact = None;
        match check_nfc() {
            Err(error) => {
                data_set.contact = None;
                afb_log_msg!(Notice, self.event,"{}",error);
                self.event.push(AuthState::Done);
            }
            Ok(value) => {
                data_set.contact = Some(value);
                self.event.push(AuthState::Done);
            }
        }
        Ok(self)
    }
}
