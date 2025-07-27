use crate::disk_arbitration::{
    DADiskCopyDescription, DADiskRef, DADiskUnmount, kDADiskDescriptionVolumeNameKey,
    kDADiskDescriptionVolumePathKey, kDADiskDescriptionVolumeUUIDKey, kDADiskUnmountOptionDefault,
};
use core_foundation::base::{CFGetTypeID, CFType, FromVoid, TCFType};
use core_foundation::dictionary::CFDictionary;
use core_foundation::string::CFString;
use core_foundation::url::CFURL;
use core_foundation::uuid::{CFUUID, CFUUIDGetUUIDBytes, CFUUIDRef};
use std::fmt::{Display, Formatter, Result};
use std::path::PathBuf;
use std::ptr;
use uuid::Uuid;

pub struct Disk {
    pub name: String,
    pub path: Option<PathBuf>,
    pub uuid: Uuid,
    ptr: DADiskRef,
}

impl Disk {
    pub fn from_ref(ptr: DADiskRef, volume_path: Option<PathBuf>) -> Option<Self> {
        if ptr.is_null() {
            return None;
        }

        let description: CFDictionary<CFString, CFType> = unsafe {
            let desc_ref = DADiskCopyDescription(ptr);
            if desc_ref.is_null() {
                return None;
            }

            CFDictionary::wrap_under_create_rule(desc_ref)
        };

        Some(Self {
            name: Disk::get_name_from_description(&description)
                .unwrap_or(String::from("Unnamed Volume")),
            path: volume_path.or_else(|| Disk::get_path_from_description(&description)),
            uuid: Disk::get_uuid_from_description(&description).unwrap_or(Uuid::nil()),
            ptr,
        })
    }

    fn get_name_from_description(description: &CFDictionary<CFString, CFType>) -> Option<String> {
        let name_key = CFString::new(kDADiskDescriptionVolumeNameKey);
        let value = description.find(&name_key)?;

        if CFString::type_id() != unsafe { CFGetTypeID(value.as_concrete_TypeRef()) } {
            log::info!("Name not found or not a CFString");
            return None;
        }

        let cf_string_ref = value.as_CFTypeRef();
        let cf_string = unsafe { CFString::from_void(cf_string_ref) };
        Some(cf_string.to_string())
    }

    fn get_path_from_description(description: &CFDictionary<CFString, CFType>) -> Option<PathBuf> {
        let path_key = CFString::new(kDADiskDescriptionVolumePathKey);
        let value = description.find(&path_key)?;

        if CFURL::type_id() != unsafe { CFGetTypeID(value.as_concrete_TypeRef()) } {
            log::info!("Path not found or not a CFURL");
            return None;
        }

        let cf_url_ref = value.as_CFTypeRef();
        let cf_url = unsafe { CFURL::from_void(cf_url_ref) };
        Some(PathBuf::from(cf_url.absolute().get_string().to_string()))
    }

    fn get_uuid_from_description(description: &CFDictionary<CFString, CFType>) -> Option<Uuid> {
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
        if let Some(path) = &self.path
            && let Some(path_str) = path.to_str()
        {
            write!(f, "{} ({}) {path_str}", self.name, self.uuid)
        } else {
            write!(f, "{} ({})", self.name, self.uuid)
        }
    }
}
