use crate::disk_arbitration::{
    DADiskCopyDescription, DADiskRef, DADiskUnmount, kDADiskDescriptionVolumeUUIDKey,
    kDADiskUnmountOptionDefault,
};
use core_foundation::base::{CFGetTypeID, CFType, TCFType};
use core_foundation::dictionary::CFDictionary;
use core_foundation::string::CFString;
use core_foundation::uuid::{CFUUID, CFUUIDGetUUIDBytes, CFUUIDRef};
use std::fmt::{Display, Formatter, Result};
use std::ptr;
use uuid::Uuid;

pub struct Disk {
    ptr: DADiskRef,
}

impl Disk {
    pub fn from_ref(ptr: DADiskRef) -> Option<Self> {
        if ptr.is_null() {
            None
        } else {
            Some(Self { ptr })
        }
    }

    pub fn get_uuid(&self) -> Option<Uuid> {
        let description: CFDictionary<CFString, CFType> = unsafe {
            let desc_ref = DADiskCopyDescription(self.ptr);
            if desc_ref.is_null() {
                return None;
            }

            CFDictionary::wrap_under_create_rule(desc_ref)
        };

        let uuid_key = CFString::new(kDADiskDescriptionVolumeUUIDKey);
        let value = description.find(&uuid_key)?;

        if CFUUID::type_id() != unsafe { CFGetTypeID(value.as_concrete_TypeRef()) } {
            log::info!("UUID not found or not a CFUUID");
            return None;
        }

        let cf_uuid_ref = value.as_CFTypeRef() as CFUUIDRef;
        let bytes = unsafe { CFUUIDGetUUIDBytes(cf_uuid_ref) };

        Some(Uuid::from_bytes([
            bytes.byte0,
            bytes.byte1,
            bytes.byte2,
            bytes.byte3,
            bytes.byte4,
            bytes.byte5,
            bytes.byte6,
            bytes.byte7,
            bytes.byte8,
            bytes.byte9,
            bytes.byte10,
            bytes.byte11,
            bytes.byte12,
            bytes.byte13,
            bytes.byte14,
            bytes.byte15,
        ]))
    }

    pub fn unmount(self) -> () {
        unsafe {
            DADiskUnmount(
                self.ptr,
                kDADiskUnmountOptionDefault,
                ptr::null_mut(),
                ptr::null_mut(),
            );
        }
    }
}

impl Display for Disk {
    fn fmt(&self, f: &mut Formatter) -> Result {
        match self.get_uuid() {
            Some(uuid) => write!(f, "Disk<{uuid}>"),
            None => write!(f, "Disk<UNKNOWN>"),
        }
    }
}
