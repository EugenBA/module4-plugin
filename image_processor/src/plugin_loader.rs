use std::ffi::{c_char, c_uint};
use libloading::{Library, Symbol};

pub struct Plugin {
    plugin: Library,
}
pub struct PluginInterface<'a> {
    pub process_image: Symbol<'a, extern "C" fn(width: c_uint,
                                                height: c_uint,
                                                rgba_data: *mut u8,
                                                params: *const c_char)>,
}

impl Plugin {
    pub fn new(filename: &str) -> Result<Self, libloading::Error> {
        Ok(Plugin {
            plugin: unsafe { Library::new(filename) }?,
        })
    }
    pub fn interface(&self) -> Result<PluginInterface<'_>, libloading::Error> {
        Ok(PluginInterface {
            // подгрузка функции по символу `trade`
            process_image: unsafe { self.plugin.get("process_image") }?,
        })
    }
}


