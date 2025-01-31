#![no_std]

extern crate alloc;
extern crate flagset;
extern crate thiserror;

pub mod bindings;

use crate::bindings as c;
use alloc::{ffi::CString, string::String};
use core::{
    marker::PhantomData,
    panic::{RefUnwindSafe, UnwindSafe},
};
use thiserror::Error;
use wut::rrc::{Rrc, RrcGuard};

static NOTIFY: Rrc = Rrc::new(
    || unsafe {
        c::NotificationModule_InitLibrary();
    },
    || unsafe {
        c::NotificationModule_DeInitLibrary();
    },
);

// region: NotificationError

#[derive(Debug, Error)]
#[repr(i32)]
pub enum NotificationError {
    #[error("")]
    ModuleNotFound = c::NotificationModuleStatus::NOTIFICATION_MODULE_RESULT_MODULE_NOT_FOUND,
    #[error("")]
    ModuleMissingExport =
        c::NotificationModuleStatus::NOTIFICATION_MODULE_RESULT_MODULE_MISSING_EXPORT,
    #[error("")]
    UnsupportedVersion =
        c::NotificationModuleStatus::NOTIFICATION_MODULE_RESULT_UNSUPPORTED_VERSION,
    #[error("")]
    InvalidArgument = c::NotificationModuleStatus::NOTIFICATION_MODULE_RESULT_INVALID_ARGUMENT,
    #[error("")]
    LibUninitialized = c::NotificationModuleStatus::NOTIFICATION_MODULE_RESULT_LIB_UNINITIALIZED,
    #[error("")]
    UnsupportedCommand =
        c::NotificationModuleStatus::NOTIFICATION_MODULE_RESULT_UNSUPPORTED_COMMAND,
    #[error("")]
    OverlayNotReady = c::NotificationModuleStatus::NOTIFICATION_MODULE_RESULT_OVERLAY_NOT_READY,
    #[error("")]
    UnsupportedType = c::NotificationModuleStatus::NOTIFICATION_MODULE_RESULT_UNSUPPORTED_TYPE,
    #[error("")]
    AllocationFailed = c::NotificationModuleStatus::NOTIFICATION_MODULE_RESULT_ALLOCATION_FAILED,
    #[error("")]
    InvalidHandle = c::NotificationModuleStatus::NOTIFICATION_MODULE_RESULT_INVALID_HANDLE,
    #[error("")]
    Unknown(i32) = c::NotificationModuleStatus::NOTIFICATION_MODULE_RESULT_UNKNOWN_ERROR,

    #[error("Internal 0-byte")]
    InternalZeroByte(#[from] alloc::ffi::NulError),
}

impl TryFrom<i32> for NotificationError {
    type Error = Self;
    fn try_from(value: i32) -> Result<Self, Self::Error> {
        use c::NotificationModuleStatus as S;
        match value {
            S::NOTIFICATION_MODULE_RESULT_SUCCESS => Ok(Self::Unknown(value)),
            S::NOTIFICATION_MODULE_RESULT_MODULE_NOT_FOUND => Err(Self::ModuleNotFound),
            S::NOTIFICATION_MODULE_RESULT_MODULE_MISSING_EXPORT => Err(Self::ModuleMissingExport),
            S::NOTIFICATION_MODULE_RESULT_UNSUPPORTED_VERSION => Err(Self::UnsupportedVersion),
            S::NOTIFICATION_MODULE_RESULT_INVALID_ARGUMENT => Err(Self::InvalidArgument),
            S::NOTIFICATION_MODULE_RESULT_LIB_UNINITIALIZED => Err(Self::LibUninitialized),
            S::NOTIFICATION_MODULE_RESULT_UNSUPPORTED_COMMAND => Err(Self::UnsupportedCommand),
            S::NOTIFICATION_MODULE_RESULT_OVERLAY_NOT_READY => Err(Self::OverlayNotReady),
            S::NOTIFICATION_MODULE_RESULT_UNSUPPORTED_TYPE => Err(Self::UnsupportedType),
            S::NOTIFICATION_MODULE_RESULT_ALLOCATION_FAILED => Err(Self::AllocationFailed),
            S::NOTIFICATION_MODULE_RESULT_INVALID_HANDLE => Err(Self::InvalidHandle),
            v => Err(Self::Unknown(v)),
        }
    }
}

// endregion

// region: Color

pub struct Color(c::NMColor);

impl Into<c::NMColor> for Color {
    fn into(self) -> c::NMColor {
        self.0
    }
}

impl From<c::NMColor> for Color {
    fn from(value: c::NMColor) -> Self {
        Self(value)
    }
}

impl Color {
    #[inline]
    pub fn white(opacity: f32) -> Self {
        return Self(c::_NMColor {
            r: 255,
            g: 255,
            b: 255,
            a: (255f32 * opacity) as u8,
        });
    }

    #[inline]
    pub fn black(opacity: f32) -> Self {
        return Self(c::_NMColor {
            r: 0,
            g: 0,
            b: 0,
            a: (255f32 * opacity) as u8,
        });
    }

    #[inline]
    pub fn red(opacity: f32) -> Self {
        return Self(c::_NMColor {
            r: 255,
            g: 0,
            b: 0,
            a: (255f32 * opacity) as u8,
        });
    }
}

// endregion

/*
pub struct NotificationBuilder {
    text: String,
    text_color: Color,
    bg_color: Color,
}

impl Default for NotificationBuilder {
    #[inline]
    fn default() -> Self {
        Self {
            text: "".to_string(),
            text_color: Color::white(1.0),
            bg_color: Color::black(0.5),
        }
    }
}

impl NotificationBuilder {
    #[inline]
    pub fn text(mut self, text: &str) -> Self {
        self.text = text.to_string();
        self
    }

    #[inline]
    pub fn text_color(mut self, color: Color) -> Self {
        self.text_color = color;
        self
    }

    #[inline]
    pub fn bg_color(mut self, color: Color) -> Self {
        self.bg_color = color;
        self
    }

    #[inline]
    pub fn build(self) -> Result<Notification, NotificationError> {
        let mut handle = Default::default();
        let text = CString::new(self.text)?;

        let status = unsafe {
            c::NotificationModule_AddDynamicNotificationEx(
                text.as_ptr(),
                &mut handle,
                self.text_color.0,
                self.bg_color.0,
                None,
                core::ptr::null_mut(),
                false,
            )
        };
        NotificationError::try_from(status)?;

        Ok(Notification {
            handle,
            _resource: NOTIFY.acquire(),
        })
    }
}

pub struct Notification {
    handle: c::NotificationModuleHandle,
    _resource: RrcGuard,
}

impl Notification {
    #[inline]
    pub fn new(text: &str) -> Result<Self, NotificationError> {
        NotificationBuilder::default().text(text).build()
    }

    #[inline]
    pub fn builder() -> NotificationBuilder {
        NotificationBuilder::default()
    }

    #[inline]
    pub fn text(&self, text: &str) -> Result<(), NotificationError> {
        let text = CString::new(text)?;

        let status = unsafe {
            c::NotificationModule_UpdateDynamicNotificationText(self.handle, text.as_ptr())
        };
        NotificationError::try_from(status)?;

        Ok(())
    }

    #[inline]
    pub fn text_color(&self, color: Color) -> Result<(), NotificationError> {
        let status = unsafe {
            c::NotificationModule_UpdateDynamicNotificationTextColor(self.handle, color.0)
        };
        NotificationError::try_from(status)?;

        Ok(())
    }

    #[inline]
    pub fn bg_color(&self, color: Color) -> Result<(), NotificationError> {
        let status = unsafe {
            c::NotificationModule_UpdateDynamicNotificationBackgroundColor(self.handle, color.0)
        };
        NotificationError::try_from(status)?;

        Ok(())
    }
}

impl Drop for Notification {
    fn drop(&mut self) {
        let status = unsafe { c::NotificationModule_FinishDynamicNotification(self.handle, 0.0) };
        NotificationError::try_from(status).unwrap();
    }
}

unsafe impl Sync for Notification {}
unsafe impl Send for Notification {}
impl RefUnwindSafe for Notification {}
impl UnwindSafe for Notification {}

pub fn info(text: &str) -> Result<(), NotificationError> {
    let _r = NOTIFY.acquire();
    let text = CString::new(text)?;
    let status = unsafe { c::NotificationModule_AddInfoNotification(text.as_ptr()) };
    NotificationError::try_from(status)?;

    Ok(())
}

unsafe extern "C" fn callback(
    handle: c::NotificationModuleHandle,
    context: *mut core::ffi::c_void,
) {
    wut::logger::init(wut::logger::Udp).unwrap();
    wut::println!("callback");
    wut::logger::deinit();
}
*/

use alloc::boxed::Box;
use core::time::Duration;

pub fn test() {
    let _r = NOTIFY.acquire();
    unsafe {
        let status = c::NotificationModule_AddErrorNotification(c"WithCallback".as_ptr());
        wut::println!("{:?}", NotificationError::try_from(status));
    };
}

// region: Notification

pub struct Notification {
    handle: c::NotificationModuleHandle,
    delay: f32,
    shake: f32,
    _resource: RrcGuard,
}

impl Notification {
    #[inline]
    pub fn text(&self, text: &str) -> Result<(), NotificationError> {
        let text = CString::new(text)?;

        let status = unsafe {
            c::NotificationModule_UpdateDynamicNotificationText(self.handle, text.as_ptr())
        };
        NotificationError::try_from(status)?;

        Ok(())
    }

    #[inline]
    pub fn text_color(&self, color: Color) -> Result<(), NotificationError> {
        let status = unsafe {
            c::NotificationModule_UpdateDynamicNotificationTextColor(self.handle, color.0)
        };
        NotificationError::try_from(status)?;

        Ok(())
    }

    #[inline]
    pub fn bg_color(&self, color: Color) -> Result<(), NotificationError> {
        let status = unsafe {
            c::NotificationModule_UpdateDynamicNotificationBackgroundColor(self.handle, color.0)
        };
        NotificationError::try_from(status)?;

        Ok(())
    }
}

impl Drop for Notification {
    fn drop(&mut self) {
        let status = unsafe {
            c::NotificationModule_FinishDynamicNotificationWithShake(
                self.handle,
                self.delay,
                self.shake,
            )
        };
        NotificationError::try_from(status).unwrap();
    }
}

unsafe impl Sync for Notification {}
unsafe impl Send for Notification {}
impl RefUnwindSafe for Notification {}
impl UnwindSafe for Notification {}

// endregion

// region: NotificationBuilder

pub struct Dynamic;
pub struct Info;
pub struct Error;

pub trait NotificationType: Sized {
    type T;
    fn show(builder: NotificationBuilder<Self>) -> Result<Self::T, NotificationError>;
}

impl NotificationType for Dynamic {
    type T = Notification;

    fn show(builder: NotificationBuilder<Self>) -> Result<Self::T, NotificationError> {
        let text = CString::new(builder.text)?;
        let callback: c::NotificationModuleNotificationFinishedCallback = match builder.callback {
            Some(_) => Some(notification_callback),
            None => None,
        };
        let context = match builder.callback {
            Some(f) => Box::into_raw(f) as *mut core::ffi::c_void,
            None => core::ptr::null_mut(),
        };

        let r = NOTIFY.acquire();
        let mut handle = c::NotificationModuleHandle::default();
        let status = unsafe {
            c::NotificationModule_AddDynamicNotificationEx(
                text.as_ptr(),
                &mut handle,
                builder.text_color.0,
                builder.background_color.0,
                callback,
                context,
                builder.keep_until_shown,
            )
        };
        NotificationError::try_from(status)?;

        Ok(Notification {
            handle,
            delay: builder.delay.map_or(0.0, |d| d.as_secs_f32()),
            shake: builder.shake.map_or(0.0, |d| d.as_secs_f32()),
            _resource: r,
        })
    }
}

impl NotificationType for Info {
    type T = ();

    fn show(builder: NotificationBuilder<Self>) -> Result<Self::T, NotificationError> {
        let text = CString::new(builder.text)?;
        let callback: c::NotificationModuleNotificationFinishedCallback = match builder.callback {
            Some(_) => Some(notification_callback),
            None => None,
        };
        let context = match builder.callback {
            Some(f) => Box::into_raw(f) as *mut core::ffi::c_void,
            None => core::ptr::null_mut(),
        };

        let _r = NOTIFY.acquire();
        let status = unsafe {
            c::NotificationModule_AddInfoNotificationEx(
                text.as_ptr(),
                builder.duration.as_secs_f32(),
                builder.text_color.0,
                builder.background_color.0,
                callback,
                context,
                builder.keep_until_shown,
            )
        };
        NotificationError::try_from(status)?;

        Ok(())
    }
}

impl NotificationType for Error {
    type T = ();

    fn show(builder: NotificationBuilder<Self>) -> Result<Self::T, NotificationError> {
        let text = CString::new(builder.text)?;
        let callback: c::NotificationModuleNotificationFinishedCallback = match builder.callback {
            Some(_) => Some(notification_callback),
            None => None,
        };
        let context = match builder.callback {
            Some(f) => Box::into_raw(f) as *mut core::ffi::c_void,
            None => core::ptr::null_mut(),
        };

        let _r = NOTIFY.acquire();
        let status = unsafe {
            c::NotificationModule_AddErrorNotificationEx(
                text.as_ptr(),
                builder.duration.as_secs_f32(),
                builder.shake.map_or(0.0, |d| d.as_secs_f32()),
                builder.text_color.0,
                builder.background_color.0,
                callback,
                context,
                builder.keep_until_shown,
            )
        };
        NotificationError::try_from(status)?;

        Ok(())
    }
}

pub struct NotificationBuilder<T: NotificationType> {
    text: String,
    duration: Duration,
    text_color: Color,
    background_color: Color,
    callback: Option<Box<Box<dyn FnOnce()>>>,
    keep_until_shown: bool,
    shake: Option<Duration>,
    delay: Option<Duration>,
    _marker: PhantomData<T>,
}

impl<T: NotificationType> Default for NotificationBuilder<T> {
    fn default() -> Self {
        Self {
            text: String::from(""),
            duration: Duration::from_secs(5),
            text_color: Color::white(1.0),
            background_color: Color::black(0.5),
            callback: None,
            keep_until_shown: true,
            shake: None,
            delay: None,
            _marker: PhantomData,
        }
    }
}

impl<T: NotificationType> NotificationBuilder<T> {
    /// Content of the notification.
    pub fn text(mut self, text: &str) -> Self {
        self.text = String::from(text);
        self
    }

    /// Time before fading out.
    pub fn duration(mut self, duration: Duration) -> Self {
        self.duration = duration;
        self
    }

    /// Text color of the Notification.
    pub fn text_color(mut self, color: Color) -> Self {
        self.text_color = color;
        self
    }

    /// Background color of the Notification.
    pub fn background_color(mut self, color: Color) -> Self {
        self.background_color = color;
        self
    }

    /// Function that will be called then the Notification fades out.
    pub fn callback<F: 'static + FnOnce()>(mut self, callback: F) -> Self {
        self.callback = Some(Box::new(Box::new(callback)));
        self
    }

    /// The Notification will be stored in a queue until can be shown.
    pub fn keep_until_shown(mut self, keep: bool) -> Self {
        self.keep_until_shown = keep;
        self
    }

    /// Queues the notification for display.
    pub fn show(self) -> Result<T::T, NotificationError> {
        T::show(self)
    }
}

impl NotificationBuilder<Dynamic> {
    pub fn shake(mut self, duration: Option<Duration>) -> Self {
        self.shake = duration;
        self
    }

    pub fn delay(mut self, duration: Option<Duration>) -> Self {
        self.delay = duration;
        self
    }
}

impl NotificationBuilder<Error> {
    pub fn shake(mut self, duration: Option<Duration>) -> Self {
        self.shake = duration;
        self
    }
}

unsafe extern "C" fn notification_callback(
    _handle: c::NotificationModuleHandle,
    arg: *mut core::ffi::c_void,
) {
    if !arg.is_null() {
        let closure = unsafe { Box::from_raw(arg as *mut Box<dyn FnOnce()>) };
        closure();
    }
}

impl<T: NotificationType> RefUnwindSafe for NotificationBuilder<T> {}
impl<T: NotificationType> UnwindSafe for NotificationBuilder<T> {}

// endregion

pub fn dynamic(text: &str) -> NotificationBuilder<Dynamic> {
    NotificationBuilder::<Dynamic>::default().text(text)
}

pub fn info(text: &str) -> NotificationBuilder<Info> {
    NotificationBuilder::<Info>::default().text(text)
}

pub fn error(text: &str) -> NotificationBuilder<Error> {
    NotificationBuilder::<Error>::default()
        .text(text)
        .background_color(Color::red(1.0))
        .shake(Some(Duration::from_secs(1)))
}
