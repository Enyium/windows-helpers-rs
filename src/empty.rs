use crate::windows;
use std::mem;

/// For structs that benefit from an alternative to `default()` to be able to write more semantic code.
pub trait Zeroed {
    fn zeroed() -> Self;
}

macro_rules! impl_zeroed {
    ($type:ty) => {
        impl Zeroed for $type {
            fn zeroed() -> Self {
                unsafe { mem::zeroed() }
            }
        }
    };
}

pub trait Null {
    const NULL: Self;
    fn is_null(&self) -> bool;
}

macro_rules! impl_null {
    ($type:ty) => {
        impl Null for $type {
            const NULL: Self = Self(0 as _);

            fn is_null(&self) -> bool {
                self.0 == 0 as _
            }
        }
    };
}

pub trait ValidateHandle {
    fn is_invalid(&self) -> bool;
}

macro_rules! impl_null_and_validate_handle {
    ($type:ty) => {
        impl_null!($type);

        impl ValidateHandle for $type {
            #[inline]
            fn is_invalid(&self) -> bool {
                <$type>::is_invalid(self)
            }
        }
    };
}

////////////////////////////////////////////////////////////////////////////////

#[cfg(feature = "f_Win32_Foundation")]
impl_zeroed!(windows::Win32::Foundation::POINT);
#[cfg(feature = "f_Win32_Foundation")]
impl_zeroed!(windows::Win32::Foundation::SIZE);

// `null()` already available, but not usable with trait bounds.
impl_null!(windows::core::PCSTR);
impl_null!(windows::core::PCWSTR);
impl_null!(windows::core::PSTR);
impl_null!(windows::core::PWSTR);

// Types without an official (trait-less) `is_invalid()` method (as of Dec. 2023).
#[cfg(feature = "f_Win32_Foundation")]
impl_null!(windows::Win32::Foundation::HWND);

// This list was built by searching for `is_invalid` in the `windows` crate documentation and textually deriving the feature names from the fully qualified types. Some features don't exist in `Cargo.toml` yet, because the crates.io feature limit means they shouldn't be added without anybody needing them.
#[cfg(feature = "f_Wdk_Storage_FileSystem_Minifilters")]
impl_null_and_validate_handle!(windows::Wdk::Storage::FileSystem::Minifilters::PFLT_CONTEXT);
#[cfg(feature = "f_Wdk_System_OfflineRegistry")]
impl_null_and_validate_handle!(windows::Wdk::System::OfflineRegistry::ORHKEY);
#[cfg(feature = "f_Win32_Devices_Bluetooth")]
impl_null_and_validate_handle!(windows::Win32::Devices::Bluetooth::HANDLE_SDP_TYPE);
#[cfg(feature = "f_Win32_Devices_Bluetooth")]
impl_null_and_validate_handle!(windows::Win32::Devices::Bluetooth::HBLUETOOTH_DEVICE_FIND);
#[cfg(feature = "f_Win32_Devices_Bluetooth")]
impl_null_and_validate_handle!(windows::Win32::Devices::Bluetooth::HBLUETOOTH_RADIO_FIND);
#[cfg(feature = "f_Win32_Devices_DeviceAndDriverInstallation")]
impl_null_and_validate_handle!(
    windows::Win32::Devices::DeviceAndDriverInstallation::HCMNOTIFICATION
);
#[cfg(feature = "f_Win32_Devices_DeviceAndDriverInstallation")]
impl_null_and_validate_handle!(windows::Win32::Devices::DeviceAndDriverInstallation::HDEVINFO);
#[cfg(feature = "f_Win32_Devices_Display")]
impl_null_and_validate_handle!(windows::Win32::Devices::Display::DHPDEV);
#[cfg(feature = "f_Win32_Devices_Display")]
impl_null_and_validate_handle!(windows::Win32::Devices::Display::DHSURF);
#[cfg(feature = "f_Win32_Devices_Display")]
impl_null_and_validate_handle!(windows::Win32::Devices::Display::HBM);
#[cfg(feature = "f_Win32_Devices_Display")]
impl_null_and_validate_handle!(windows::Win32::Devices::Display::HDEV);
#[cfg(feature = "f_Win32_Devices_Display")]
impl_null_and_validate_handle!(windows::Win32::Devices::Display::HDRVOBJ);
#[cfg(feature = "f_Win32_Devices_Display")]
impl_null_and_validate_handle!(windows::Win32::Devices::Display::HFASTMUTEX);
#[cfg(feature = "f_Win32_Devices_Display")]
impl_null_and_validate_handle!(windows::Win32::Devices::Display::HSEMAPHORE);
#[cfg(feature = "f_Win32_Devices_Display")]
impl_null_and_validate_handle!(windows::Win32::Devices::Display::HSURF);
#[cfg(feature = "f_Win32_Devices_Enumeration_Pnp")]
impl_null_and_validate_handle!(windows::Win32::Devices::Enumeration::Pnp::HSWDEVICE);
#[cfg(feature = "f_Win32_Devices_SerialCommunication")]
impl_null_and_validate_handle!(windows::Win32::Devices::SerialCommunication::HCOMDB);
#[cfg(feature = "f_Win32_Devices_Usb")]
impl_null_and_validate_handle!(windows::Win32::Devices::Usb::WINUSB_INTERFACE_HANDLE);
#[cfg(feature = "f_Win32_Foundation")]
impl_null_and_validate_handle!(windows::Win32::Foundation::HANDLE);
#[cfg(feature = "f_Win32_Foundation")]
impl_null_and_validate_handle!(windows::Win32::Foundation::HGLOBAL);
#[cfg(not(feature = "windows_v0_48"))]
#[cfg(feature = "f_Win32_Foundation")]
impl_null_and_validate_handle!(windows::Win32::Foundation::HINSTANCE);
#[cfg(feature = "f_Win32_Foundation")]
impl_null_and_validate_handle!(windows::Win32::Foundation::HLOCAL);
#[cfg(feature = "f_Win32_Foundation")]
impl_null_and_validate_handle!(windows::Win32::Foundation::HMODULE);
#[cfg(feature = "f_Win32_Foundation")]
impl_null_and_validate_handle!(windows::Win32::Foundation::HRSRC);
#[cfg(feature = "f_Win32_Foundation")]
impl_null_and_validate_handle!(windows::Win32::Foundation::PSID);
#[cfg(feature = "f_Win32_Globalization")]
impl_null_and_validate_handle!(windows::Win32::Globalization::HIMC);
#[cfg(feature = "f_Win32_Globalization")]
impl_null_and_validate_handle!(windows::Win32::Globalization::HIMCC);
#[cfg(feature = "f_Win32_Globalization")]
impl_null_and_validate_handle!(windows::Win32::Globalization::HSAVEDUILANGUAGES);
#[cfg(feature = "f_Win32_Graphics_Gdi")]
impl_null_and_validate_handle!(windows::Win32::Graphics::Gdi::HBITMAP);
#[cfg(feature = "f_Win32_Graphics_Gdi")]
impl_null_and_validate_handle!(windows::Win32::Graphics::Gdi::HBRUSH);
#[cfg(feature = "f_Win32_Graphics_Gdi")]
impl_null_and_validate_handle!(windows::Win32::Graphics::Gdi::HDC);
#[cfg(feature = "f_Win32_Graphics_Gdi")]
impl_null_and_validate_handle!(windows::Win32::Graphics::Gdi::HENHMETAFILE);
#[cfg(feature = "f_Win32_Graphics_Gdi")]
impl_null_and_validate_handle!(windows::Win32::Graphics::Gdi::HFONT);
#[cfg(feature = "f_Win32_Graphics_Gdi")]
impl_null_and_validate_handle!(windows::Win32::Graphics::Gdi::HGDIOBJ);
#[cfg(feature = "f_Win32_Graphics_Gdi")]
impl_null_and_validate_handle!(windows::Win32::Graphics::Gdi::HMETAFILE);
#[cfg(feature = "f_Win32_Graphics_Gdi")]
impl_null_and_validate_handle!(windows::Win32::Graphics::Gdi::HMONITOR);
#[cfg(feature = "f_Win32_Graphics_Gdi")]
impl_null_and_validate_handle!(windows::Win32::Graphics::Gdi::HPALETTE);
#[cfg(feature = "f_Win32_Graphics_Gdi")]
impl_null_and_validate_handle!(windows::Win32::Graphics::Gdi::HPEN);
#[cfg(feature = "f_Win32_Graphics_Gdi")]
impl_null_and_validate_handle!(windows::Win32::Graphics::Gdi::HRGN);
#[cfg(feature = "f_Win32_Graphics_OpenGL")]
impl_null_and_validate_handle!(windows::Win32::Graphics::OpenGL::HGLRC);
#[cfg(feature = "f_Win32_Media_Audio")]
impl_null_and_validate_handle!(windows::Win32::Media::Audio::HACMDRIVER);
#[cfg(feature = "f_Win32_Media_Audio")]
impl_null_and_validate_handle!(windows::Win32::Media::Audio::HACMDRIVERID);
#[cfg(feature = "f_Win32_Media_Audio")]
impl_null_and_validate_handle!(windows::Win32::Media::Audio::HACMOBJ);
#[cfg(feature = "f_Win32_Media_Audio")]
impl_null_and_validate_handle!(windows::Win32::Media::Audio::HACMSTREAM);
#[cfg(feature = "f_Win32_Media_Audio")]
impl_null_and_validate_handle!(windows::Win32::Media::Audio::HMIDI);
#[cfg(feature = "f_Win32_Media_Audio")]
impl_null_and_validate_handle!(windows::Win32::Media::Audio::HMIDIIN);
#[cfg(feature = "f_Win32_Media_Audio")]
impl_null_and_validate_handle!(windows::Win32::Media::Audio::HMIDIOUT);
#[cfg(feature = "f_Win32_Media_Audio")]
impl_null_and_validate_handle!(windows::Win32::Media::Audio::HMIDISTRM);
#[cfg(feature = "f_Win32_Media_Audio")]
impl_null_and_validate_handle!(windows::Win32::Media::Audio::HMIXER);
#[cfg(feature = "f_Win32_Media_Audio")]
impl_null_and_validate_handle!(windows::Win32::Media::Audio::HMIXEROBJ);
#[cfg(feature = "f_Win32_Media_Audio")]
impl_null_and_validate_handle!(windows::Win32::Media::Audio::HWAVE);
#[cfg(feature = "f_Win32_Media_Audio")]
impl_null_and_validate_handle!(windows::Win32::Media::Audio::HWAVEIN);
#[cfg(feature = "f_Win32_Media_Audio")]
impl_null_and_validate_handle!(windows::Win32::Media::Audio::HWAVEOUT);
#[cfg(feature = "f_Win32_Media")]
impl_null_and_validate_handle!(windows::Win32::Media::HTASK);
#[cfg(feature = "f_Win32_Media_Multimedia")]
impl_null_and_validate_handle!(windows::Win32::Media::Multimedia::HDRVR);
#[cfg(feature = "f_Win32_Media_Multimedia")]
impl_null_and_validate_handle!(windows::Win32::Media::Multimedia::HIC);
#[cfg(feature = "f_Win32_Media_Multimedia")]
impl_null_and_validate_handle!(windows::Win32::Media::Multimedia::HMMIO);
#[cfg(feature = "f_Win32_Media_Multimedia")]
impl_null_and_validate_handle!(windows::Win32::Media::Multimedia::HVIDEO);
#[cfg(feature = "f_Win32_Media_Speech")]
impl_null_and_validate_handle!(windows::Win32::Media::Speech::SPGRAMMARHANDLE);
#[cfg(feature = "f_Win32_Media_Speech")]
impl_null_and_validate_handle!(windows::Win32::Media::Speech::SPPHRASEPROPERTYHANDLE);
#[cfg(feature = "f_Win32_Media_Speech")]
impl_null_and_validate_handle!(windows::Win32::Media::Speech::SPPHRASERULEHANDLE);
#[cfg(feature = "f_Win32_Media_Speech")]
impl_null_and_validate_handle!(windows::Win32::Media::Speech::SPRECOCONTEXTHANDLE);
#[cfg(feature = "f_Win32_Media_Speech")]
impl_null_and_validate_handle!(windows::Win32::Media::Speech::SPRULEHANDLE);
#[cfg(feature = "f_Win32_Media_Speech")]
impl_null_and_validate_handle!(windows::Win32::Media::Speech::SPSTATEHANDLE);
#[cfg(feature = "f_Win32_Media_Speech")]
impl_null_and_validate_handle!(windows::Win32::Media::Speech::SPTRANSITIONID);
#[cfg(feature = "f_Win32_Media_Speech")]
impl_null_and_validate_handle!(windows::Win32::Media::Speech::SPWORDHANDLE);
#[cfg(feature = "f_Win32_Networking_ActiveDirectory")]
impl_null_and_validate_handle!(windows::Win32::Networking::ActiveDirectory::ADS_SEARCH_HANDLE);
#[cfg(feature = "f_Win32_Networking_WebSocket")]
impl_null_and_validate_handle!(windows::Win32::Networking::WebSocket::WEB_SOCKET_HANDLE);
#[cfg(feature = "f_Win32_Networking_WinInet")]
impl_null_and_validate_handle!(windows::Win32::Networking::WinInet::HTTP_PUSH_WAIT_HANDLE);
#[cfg(feature = "f_Win32_Networking_WinSock")]
impl_null_and_validate_handle!(windows::Win32::Networking::WinSock::WSAEVENT);
#[cfg(feature = "f_Win32_NetworkManagement_IpHelper")]
impl_null_and_validate_handle!(windows::Win32::NetworkManagement::IpHelper::HIFTIMESTAMPCHANGE);
#[cfg(feature = "f_Win32_NetworkManagement_QoS")]
impl_null_and_validate_handle!(windows::Win32::NetworkManagement::QoS::LPM_HANDLE);
#[cfg(feature = "f_Win32_NetworkManagement_QoS")]
impl_null_and_validate_handle!(windows::Win32::NetworkManagement::QoS::RHANDLE);
#[cfg(feature = "f_Win32_NetworkManagement_Rras")]
impl_null_and_validate_handle!(windows::Win32::NetworkManagement::Rras::HRASCONN);
#[cfg(feature = "f_Win32_Security_Authentication_Identity")]
impl_null_and_validate_handle!(windows::Win32::Security::Authentication::Identity::LSA_HANDLE);
#[cfg(feature = "f_Win32_Security_Authorization")]
impl_null_and_validate_handle!(
    windows::Win32::Security::Authorization::AUTHZ_ACCESS_CHECK_RESULTS_HANDLE
);
#[cfg(feature = "f_Win32_Security_Authorization")]
impl_null_and_validate_handle!(windows::Win32::Security::Authorization::AUTHZ_AUDIT_EVENT_HANDLE);
#[cfg(feature = "f_Win32_Security_Authorization")]
impl_null_and_validate_handle!(
    windows::Win32::Security::Authorization::AUTHZ_AUDIT_EVENT_TYPE_HANDLE
);
#[cfg(feature = "f_Win32_Security_Authorization")]
impl_null_and_validate_handle!(
    windows::Win32::Security::Authorization::AUTHZ_CAP_CHANGE_SUBSCRIPTION_HANDLE
);
#[cfg(feature = "f_Win32_Security_Authorization")]
impl_null_and_validate_handle!(
    windows::Win32::Security::Authorization::AUTHZ_CLIENT_CONTEXT_HANDLE
);
#[cfg(feature = "f_Win32_Security_Authorization")]
impl_null_and_validate_handle!(
    windows::Win32::Security::Authorization::AUTHZ_RESOURCE_MANAGER_HANDLE
);
#[cfg(feature = "f_Win32_Security_Authorization")]
impl_null_and_validate_handle!(
    windows::Win32::Security::Authorization::AUTHZ_SECURITY_EVENT_PROVIDER_HANDLE
);
#[cfg(feature = "f_Win32_Security_Cryptography")]
impl_null_and_validate_handle!(windows::Win32::Security::Cryptography::BCRYPT_ALG_HANDLE);
#[cfg(feature = "f_Win32_Security_Cryptography")]
impl_null_and_validate_handle!(windows::Win32::Security::Cryptography::BCRYPT_HANDLE);
#[cfg(feature = "f_Win32_Security_Cryptography")]
impl_null_and_validate_handle!(windows::Win32::Security::Cryptography::BCRYPT_HASH_HANDLE);
#[cfg(feature = "f_Win32_Security_Cryptography")]
impl_null_and_validate_handle!(windows::Win32::Security::Cryptography::BCRYPT_KEY_HANDLE);
#[cfg(feature = "f_Win32_Security_Cryptography")]
impl_null_and_validate_handle!(windows::Win32::Security::Cryptography::BCRYPT_SECRET_HANDLE);
#[cfg(feature = "f_Win32_Security_Cryptography")]
impl_null_and_validate_handle!(windows::Win32::Security::Cryptography::HCERTCHAINENGINE);
#[cfg(feature = "f_Win32_Security_Cryptography")]
impl_null_and_validate_handle!(windows::Win32::Security::Cryptography::HCERTSTORE);
#[cfg(feature = "f_Win32_Security_Cryptography")]
impl_null_and_validate_handle!(windows::Win32::Security::Cryptography::HCERTSTOREPROV);
#[cfg(feature = "f_Win32_Security_Cryptography")]
impl_null_and_validate_handle!(windows::Win32::Security::Cryptography::HCRYPTASYNC);
#[cfg(feature = "f_Win32_Security_Cryptography")]
impl_null_and_validate_handle!(windows::Win32::Security::Cryptography::HCRYPTPROV_LEGACY);
#[cfg(feature = "f_Win32_Security_Cryptography")]
impl_null_and_validate_handle!(
    windows::Win32::Security::Cryptography::HCRYPTPROV_OR_NCRYPT_KEY_HANDLE
);
#[cfg(feature = "f_Win32_Security_Cryptography")]
impl_null_and_validate_handle!(windows::Win32::Security::Cryptography::NCRYPT_HANDLE);
#[cfg(feature = "f_Win32_Security_Cryptography")]
impl_null_and_validate_handle!(windows::Win32::Security::Cryptography::NCRYPT_HASH_HANDLE);
#[cfg(feature = "f_Win32_Security_Cryptography")]
impl_null_and_validate_handle!(windows::Win32::Security::Cryptography::NCRYPT_KEY_HANDLE);
#[cfg(feature = "f_Win32_Security_Cryptography")]
impl_null_and_validate_handle!(windows::Win32::Security::Cryptography::NCRYPT_PROV_HANDLE);
#[cfg(feature = "f_Win32_Security_Cryptography")]
impl_null_and_validate_handle!(windows::Win32::Security::Cryptography::NCRYPT_SECRET_HANDLE);
#[cfg(feature = "f_Win32_Security")]
impl_null_and_validate_handle!(windows::Win32::Security::HDIAGNOSTIC_DATA_QUERY_SESSION);
#[cfg(feature = "f_Win32_Security")]
impl_null_and_validate_handle!(windows::Win32::Security::HDIAGNOSTIC_EVENT_CATEGORY_DESCRIPTION);
#[cfg(feature = "f_Win32_Security")]
impl_null_and_validate_handle!(windows::Win32::Security::HDIAGNOSTIC_EVENT_PRODUCER_DESCRIPTION);
#[cfg(feature = "f_Win32_Security")]
impl_null_and_validate_handle!(windows::Win32::Security::HDIAGNOSTIC_EVENT_TAG_DESCRIPTION);
#[cfg(feature = "f_Win32_Security")]
impl_null_and_validate_handle!(windows::Win32::Security::HDIAGNOSTIC_RECORD);
#[cfg(feature = "f_Win32_Security")]
impl_null_and_validate_handle!(windows::Win32::Security::HDIAGNOSTIC_REPORT);
#[cfg(feature = "f_Win32_Security")]
impl_null_and_validate_handle!(windows::Win32::Security::NCRYPT_DESCRIPTOR_HANDLE);
#[cfg(feature = "f_Win32_Security")]
impl_null_and_validate_handle!(windows::Win32::Security::NCRYPT_STREAM_HANDLE);
#[cfg(feature = "f_Win32_Security")]
impl_null_and_validate_handle!(windows::Win32::Security::PSECURITY_DESCRIPTOR);
#[cfg(feature = "f_Win32_Security")]
impl_null_and_validate_handle!(windows::Win32::Security::SAFER_LEVEL_HANDLE);
#[cfg(feature = "f_Win32_Security")]
impl_null_and_validate_handle!(windows::Win32::Security::SC_HANDLE);
#[cfg(feature = "f_Win32_Storage_CloudFilters")]
impl_null_and_validate_handle!(windows::Win32::Storage::CloudFilters::CF_CONNECTION_KEY);
#[cfg(feature = "f_Win32_Storage_Compression")]
impl_null_and_validate_handle!(windows::Win32::Storage::Compression::COMPRESSOR_HANDLE);
#[cfg(feature = "f_Win32_Storage_InstallableFileSystems")]
impl_null_and_validate_handle!(windows::Win32::Storage::InstallableFileSystems::HFILTER_INSTANCE);
#[cfg(feature = "f_Win32_Storage_InstallableFileSystems")]
impl_null_and_validate_handle!(windows::Win32::Storage::InstallableFileSystems::HFILTER);
#[cfg(feature = "f_Win32_Storage_Jet")]
impl_null_and_validate_handle!(windows::Win32::Storage::Jet::JET_LS);
#[cfg(feature = "f_Win32_Storage_Jet")]
impl_null_and_validate_handle!(windows::Win32::Storage::Jet::JET_OSSNAPID);
#[cfg(feature = "f_Win32_Storage_ProjectedFileSystem")]
impl_null_and_validate_handle!(
    windows::Win32::Storage::ProjectedFileSystem::PRJ_DIR_ENTRY_BUFFER_HANDLE
);
#[cfg(feature = "f_Win32_Storage_ProjectedFileSystem")]
impl_null_and_validate_handle!(
    windows::Win32::Storage::ProjectedFileSystem::PRJ_NAMESPACE_VIRTUALIZATION_CONTEXT
);
#[cfg(feature = "f_Win32_Storage_StructuredStorage")]
impl_null_and_validate_handle!(windows::Win32::Storage::StructuredStorage::JET_API_PTR);
#[cfg(feature = "f_Win32_Storage_StructuredStorage")]
impl_null_and_validate_handle!(windows::Win32::Storage::StructuredStorage::JET_HANDLE);
#[cfg(feature = "f_Win32_Storage_StructuredStorage")]
impl_null_and_validate_handle!(windows::Win32::Storage::StructuredStorage::JET_INSTANCE);
#[cfg(feature = "f_Win32_Storage_StructuredStorage")]
impl_null_and_validate_handle!(windows::Win32::Storage::StructuredStorage::JET_SESID);
#[cfg(feature = "f_Win32_Storage_StructuredStorage")]
impl_null_and_validate_handle!(windows::Win32::Storage::StructuredStorage::JET_TABLEID);
#[cfg(feature = "f_Win32_Storage_Xps")]
impl_null_and_validate_handle!(windows::Win32::Storage::Xps::HPTPROVIDER);
#[cfg(feature = "f_Win32_System_Antimalware")]
impl_null_and_validate_handle!(windows::Win32::System::Antimalware::HAMSICONTEXT);
#[cfg(feature = "f_Win32_System_Antimalware")]
impl_null_and_validate_handle!(windows::Win32::System::Antimalware::HAMSISESSION);
#[cfg(feature = "f_Win32_System_ApplicationInstallationAndServicing")]
impl_null_and_validate_handle!(
    windows::Win32::System::ApplicationInstallationAndServicing::MSIHANDLE
);
#[cfg(feature = "f_Win32_System_Com")]
impl_null_and_validate_handle!(windows::Win32::System::Com::CO_DEVICE_CATALOG_COOKIE);
#[cfg(feature = "f_Win32_System_Com")]
impl_null_and_validate_handle!(windows::Win32::System::Com::CO_MTA_USAGE_COOKIE);
#[cfg(feature = "f_Win32_System_Console")]
impl_null_and_validate_handle!(windows::Win32::System::Console::HPCON);
#[cfg(feature = "f_Win32_System_DataExchange")]
impl_null_and_validate_handle!(windows::Win32::System::DataExchange::HCONV);
#[cfg(feature = "f_Win32_System_DataExchange")]
impl_null_and_validate_handle!(windows::Win32::System::DataExchange::HCONVLIST);
#[cfg(feature = "f_Win32_System_DataExchange")]
impl_null_and_validate_handle!(windows::Win32::System::DataExchange::HDDEDATA);
#[cfg(feature = "f_Win32_System_DataExchange")]
impl_null_and_validate_handle!(windows::Win32::System::DataExchange::HSZ);
#[cfg(feature = "f_Win32_System_Diagnostics_Etw")]
impl_null_and_validate_handle!(windows::Win32::System::Diagnostics::Etw::TDH_HANDLE);
#[cfg(feature = "f_Win32_System_Diagnostics_ProcessSnapshotting")]
impl_null_and_validate_handle!(windows::Win32::System::Diagnostics::ProcessSnapshotting::HPSS);
#[cfg(feature = "f_Win32_System_Diagnostics_ProcessSnapshotting")]
impl_null_and_validate_handle!(windows::Win32::System::Diagnostics::ProcessSnapshotting::HPSSWALK);
#[cfg(feature = "f_Win32_System_ErrorReporting")]
impl_null_and_validate_handle!(windows::Win32::System::ErrorReporting::HREPORT);
#[cfg(feature = "f_Win32_System_ErrorReporting")]
impl_null_and_validate_handle!(windows::Win32::System::ErrorReporting::HREPORTSTORE);
#[cfg(feature = "f_Win32_System_EventLog")]
impl_null_and_validate_handle!(windows::Win32::System::EventLog::EVT_HANDLE);
#[cfg(feature = "f_Win32_System_HostCompute")]
impl_null_and_validate_handle!(windows::Win32::System::HostCompute::HCS_CALLBACK);
#[cfg(feature = "f_Win32_System_HostComputeSystem")]
impl_null_and_validate_handle!(windows::Win32::System::HostComputeSystem::HCS_OPERATION);
#[cfg(feature = "f_Win32_System_HostComputeSystem")]
impl_null_and_validate_handle!(windows::Win32::System::HostComputeSystem::HCS_PROCESS);
#[cfg(feature = "f_Win32_System_HostComputeSystem")]
impl_null_and_validate_handle!(windows::Win32::System::HostComputeSystem::HCS_SYSTEM);
#[cfg(feature = "f_Win32_System_Hypervisor")]
impl_null_and_validate_handle!(windows::Win32::System::Hypervisor::WHV_PARTITION_HANDLE);
#[cfg(feature = "f_Win32_System_Iis")]
impl_null_and_validate_handle!(windows::Win32::System::Iis::HCONN);
#[cfg(feature = "f_Win32_System_Ole")]
impl_null_and_validate_handle!(windows::Win32::System::Ole::OLE_HANDLE);
#[cfg(feature = "f_Win32_System_Power")]
impl_null_and_validate_handle!(windows::Win32::System::Power::HPOWERNOTIFY);
#[cfg(feature = "f_Win32_System_Registry")]
impl_null_and_validate_handle!(windows::Win32::System::Registry::HKEY);
#[cfg(feature = "f_Win32_System_Search")]
impl_null_and_validate_handle!(windows::Win32::System::Search::HACCESSOR);
#[cfg(feature = "f_Win32_System_Services")]
impl_null_and_validate_handle!(windows::Win32::System::Services::SERVICE_STATUS_HANDLE);
#[cfg(feature = "f_Win32_System_StationsAndDesktops")]
impl_null_and_validate_handle!(windows::Win32::System::StationsAndDesktops::HDESK);
#[cfg(feature = "f_Win32_System_StationsAndDesktops")]
impl_null_and_validate_handle!(windows::Win32::System::StationsAndDesktops::HWINSTA);
#[cfg(feature = "f_Win32_System_Threading")]
impl_null_and_validate_handle!(windows::Win32::System::Threading::LPPROC_THREAD_ATTRIBUTE_LIST);
#[cfg(feature = "f_Win32_System_Threading")]
impl_null_and_validate_handle!(windows::Win32::System::Threading::PTP_CALLBACK_INSTANCE);
#[cfg(feature = "f_Win32_System_Threading")]
impl_null_and_validate_handle!(windows::Win32::System::Threading::PTP_IO);
#[cfg(feature = "f_Win32_System_Threading")]
impl_null_and_validate_handle!(windows::Win32::System::Threading::PTP_TIMER);
#[cfg(feature = "f_Win32_System_Threading")]
impl_null_and_validate_handle!(windows::Win32::System::Threading::PTP_WAIT);
#[cfg(feature = "f_Win32_System_Threading")]
impl_null_and_validate_handle!(windows::Win32::System::Threading::PTP_WORK);
#[cfg(feature = "f_Win32_System_WindowsProgramming")]
impl_null_and_validate_handle!(
    windows::Win32::System::WindowsProgramming::FEATURE_STATE_CHANGE_SUBSCRIPTION
);
#[cfg(feature = "f_Win32_System_WindowsProgramming")]
impl_null_and_validate_handle!(windows::Win32::System::WindowsProgramming::FH_SERVICE_PIPE_HANDLE);
#[cfg(feature = "f_Win32_System_WindowsProgramming")]
impl_null_and_validate_handle!(windows::Win32::System::WindowsProgramming::HWINWATCH);
#[cfg(feature = "f_Win32_System_WinRT")]
impl_null_and_validate_handle!(
    windows::Win32::System::WinRT::APARTMENT_SHUTDOWN_REGISTRATION_COOKIE
);
#[cfg(feature = "f_Win32_System_WinRT")]
impl_null_and_validate_handle!(windows::Win32::System::WinRT::HSTRING_BUFFER);
#[cfg(feature = "f_Win32_System_WinRT")]
impl_null_and_validate_handle!(windows::Win32::System::WinRT::ROPARAMIIDHANDLE);
#[cfg(feature = "f_Win32_UI_Accessibility")]
impl_null_and_validate_handle!(windows::Win32::UI::Accessibility::HUIAEVENT);
#[cfg(feature = "f_Win32_UI_Accessibility")]
impl_null_and_validate_handle!(windows::Win32::UI::Accessibility::HUIANODE);
#[cfg(feature = "f_Win32_UI_Accessibility")]
impl_null_and_validate_handle!(windows::Win32::UI::Accessibility::HUIAPATTERNOBJECT);
#[cfg(feature = "f_Win32_UI_Accessibility")]
impl_null_and_validate_handle!(windows::Win32::UI::Accessibility::HUIATEXTRANGE);
#[cfg(feature = "f_Win32_UI_Accessibility")]
impl_null_and_validate_handle!(windows::Win32::UI::Accessibility::HWINEVENTHOOK);
#[cfg(feature = "f_Win32_UI_ColorSystem")]
impl_null_and_validate_handle!(windows::Win32::UI::ColorSystem::HCOLORSPACE);
#[cfg(feature = "f_Win32_UI_Controls")]
impl_null_and_validate_handle!(windows::Win32::UI::Controls::HDPA);
#[cfg(feature = "f_Win32_UI_Controls")]
impl_null_and_validate_handle!(windows::Win32::UI::Controls::HDSA);
#[cfg(feature = "f_Win32_UI_Controls")]
impl_null_and_validate_handle!(windows::Win32::UI::Controls::HIMAGELIST);
#[cfg(feature = "f_Win32_UI_Controls")]
impl_null_and_validate_handle!(windows::Win32::UI::Controls::HPROPSHEETPAGE);
#[cfg(feature = "f_Win32_UI_Controls")]
impl_null_and_validate_handle!(windows::Win32::UI::Controls::HSYNTHETICPOINTERDEVICE);
#[cfg(feature = "f_Win32_UI_Controls")]
impl_null_and_validate_handle!(windows::Win32::UI::Controls::HTHEME);
#[cfg(feature = "f_Win32_UI_HiDpi")]
impl_null_and_validate_handle!(windows::Win32::UI::HiDpi::DPI_AWARENESS_CONTEXT);
#[cfg(feature = "f_Win32_UI_Input")]
impl_null_and_validate_handle!(windows::Win32::UI::Input::HRAWINPUT);
#[cfg(feature = "f_Win32_UI_Input_Touch")]
impl_null_and_validate_handle!(windows::Win32::UI::Input::Touch::HGESTUREINFO);
#[cfg(feature = "f_Win32_UI_Input_Touch")]
impl_null_and_validate_handle!(windows::Win32::UI::Input::Touch::HTOUCHINPUT);
#[cfg(feature = "f_Win32_UI_InteractionContext")]
impl_null_and_validate_handle!(windows::Win32::UI::InteractionContext::HINTERACTIONCONTEXT);
#[cfg(feature = "f_Win32_UI_Shell")]
impl_null_and_validate_handle!(windows::Win32::UI::Shell::HDROP);
#[cfg(feature = "f_Win32_UI_Shell")]
impl_null_and_validate_handle!(windows::Win32::UI::Shell::HPSXA);
#[cfg(feature = "f_Win32_UI_TabletPC")]
impl_null_and_validate_handle!(windows::Win32::UI::TabletPC::HRECOALT);
#[cfg(feature = "f_Win32_UI_TabletPC")]
impl_null_and_validate_handle!(windows::Win32::UI::TabletPC::HRECOCONTEXT);
#[cfg(feature = "f_Win32_UI_TabletPC")]
impl_null_and_validate_handle!(windows::Win32::UI::TabletPC::HRECOGNIZER);
#[cfg(feature = "f_Win32_UI_TabletPC")]
impl_null_and_validate_handle!(windows::Win32::UI::TabletPC::HRECOLATTICE);
#[cfg(feature = "f_Win32_UI_TabletPC")]
impl_null_and_validate_handle!(windows::Win32::UI::TabletPC::HRECOWORDLIST);
#[cfg(feature = "f_Win32_UI_TextServices")]
impl_null_and_validate_handle!(windows::Win32::UI::TextServices::HKL);
#[cfg(feature = "f_Win32_UI_WindowsAndMessaging")]
impl_null_and_validate_handle!(windows::Win32::UI::WindowsAndMessaging::HACCEL);
#[cfg(feature = "f_Win32_UI_WindowsAndMessaging")]
impl_null_and_validate_handle!(windows::Win32::UI::WindowsAndMessaging::HCURSOR);
#[cfg(not(feature = "windows_v0_48"))]
#[cfg(feature = "f_Win32_UI_WindowsAndMessaging")]
impl_null_and_validate_handle!(windows::Win32::UI::WindowsAndMessaging::HDEVNOTIFY);
#[cfg(feature = "f_Win32_UI_WindowsAndMessaging")]
impl_null_and_validate_handle!(windows::Win32::UI::WindowsAndMessaging::HDWP);
#[cfg(feature = "f_Win32_UI_WindowsAndMessaging")]
impl_null_and_validate_handle!(windows::Win32::UI::WindowsAndMessaging::HHOOK);
#[cfg(feature = "f_Win32_UI_WindowsAndMessaging")]
impl_null_and_validate_handle!(windows::Win32::UI::WindowsAndMessaging::HICON);
#[cfg(feature = "f_Win32_UI_WindowsAndMessaging")]
impl_null_and_validate_handle!(windows::Win32::UI::WindowsAndMessaging::HMENU);
