extern crate cocoa;
extern crate objc;
extern crate plist;

use cocoa::base::{nil, YES};
use cocoa::foundation::{NSArray, NSAutoreleasePool, NSString};
use objc::runtime::{Class, Object};
use objc::{msg_send, sel, sel_impl};
use plist::Value;
use std::ffi::CStr;
use std::fs;
use std::path::{Path, PathBuf};

fn main() {
    let _pool = unsafe { NSAutoreleasePool::new(nil) };

    list_applications("/Applications");
    list_applications("/System/Applications");
    list_applications("/System/Library/CoreServices");

    unsafe {
        let workspace: *mut Object = msg_send![Class::get("NSWorkspace").unwrap(), sharedWorkspace];
        let app_name = NSString::alloc(nil).init_str("Safari");
        let result: bool = msg_send![workspace, launchApplication: app_name];
        if result {
            println!("Safari launched successfully!");
        } else {
            println!("Failed to launch Safari.");
        }
    }
}

fn list_applications(dir: &str) {
    let path = Path::new(dir);
    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries {
            if let Ok(entry) = entry {
                let path = entry.path();
                if path.is_dir() && path.extension().and_then(|s| s.to_str()) == Some("app") {
                    if let Some((name, icon_path)) = get_app_info(&path) {
                        println!("App: {}, Icon: {:?}", name, icon_path);
                    }
                }
            }
        }
    }
}

fn get_app_info(app_path: &Path) -> Option<(String, PathBuf)> {
    let info_plist_path = app_path.join("Contents/Info.plist");
    if let Ok(plist) = plist::Value::from_file(&info_plist_path) {
        if let Value::Dictionary(dict) = plist {
            if let Some(Value::String(name)) = dict.get("CFBundleName") {
                let icon_file = dict.get("CFBundleIconFile").and_then(|value| {
                    if let Value::String(icon_name) = value {
                        Some(icon_name.clone())
                    } else {
                        None
                    }
                });

                if let Some(icon_file) = icon_file {
                    let icon_path = app_path
                        .join("Contents/Resources")
                        .join(icon_file)
                        .with_extension("icns");
                    return Some((name.clone(), icon_path));
                }
            }
        }
    }
    None
}
