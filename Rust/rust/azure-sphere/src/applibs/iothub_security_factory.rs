use azure_sphere_sys::applibs::iothub_security_factory;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum SecurityType {
    Unknown,
    Sas,
    X509,
    HTTPEdge,
    SymmetricKey,
}

pub struct IotHubSecurityFactory {}

impl IotHubSecurityFactory {
    pub fn init(security_type: SecurityType) -> Result<Self, i32> {
        let security_type = match security_type {
            SecurityType::Unknown => {
                iothub_security_factory::IOTHUB_SECURITY_TYPE_TAG_IOTHUB_SECURITY_TYPE_UNKNOWN
            }
            SecurityType::Sas => {
                iothub_security_factory::IOTHUB_SECURITY_TYPE_TAG_IOTHUB_SECURITY_TYPE_SAS
            }
            SecurityType::X509 => {
                iothub_security_factory::IOTHUB_SECURITY_TYPE_TAG_IOTHUB_SECURITY_TYPE_X509
            }
            SecurityType::HTTPEdge => {
                iothub_security_factory::IOTHUB_SECURITY_TYPE_TAG_IOTHUB_SECURITY_TYPE_HTTP_EDGE
            }
            SecurityType::SymmetricKey => {
                iothub_security_factory::IOTHUB_SECURITY_TYPE_TAG_IOTHUB_SECURITY_TYPE_SYMMETRIC_KEY
            }
        };
        // The C function returns an int which may be 0 or MU_FAILURE, which is only
        // known to be nonzero.  It may be a line # in a source file, for example.
        let ret = unsafe { iothub_security_factory::iothub_security_init(security_type) };
        if ret == 0 {
            Ok(Self {})
        } else {
            Err(ret)
        }
    }

    pub fn security_type() -> SecurityType {
        let ret = unsafe { iothub_security_factory::iothub_security_type() };
        match ret {
            iothub_security_factory::IOTHUB_SECURITY_TYPE_TAG_IOTHUB_SECURITY_TYPE_SAS => SecurityType::Sas,
            iothub_security_factory::IOTHUB_SECURITY_TYPE_TAG_IOTHUB_SECURITY_TYPE_X509 => SecurityType::X509,
            iothub_security_factory::IOTHUB_SECURITY_TYPE_TAG_IOTHUB_SECURITY_TYPE_HTTP_EDGE => SecurityType::HTTPEdge,
            iothub_security_factory::IOTHUB_SECURITY_TYPE_TAG_IOTHUB_SECURITY_TYPE_SYMMETRIC_KEY => SecurityType::SymmetricKey,
            _ => SecurityType::Unknown,
        }
    }

    // iothub_security_interface() is declared but not defined.  Ignored.
}

impl Drop for IotHubSecurityFactory {
    fn drop(&mut self) {
        unsafe { iothub_security_factory::iothub_security_deinit() }
    }
}
