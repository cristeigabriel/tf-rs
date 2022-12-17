//! Module for project modules

use crate::{error::Error, memory::generic_address::GenericAddress, read_c_string, GenericErrOr};
use ntapi::winapi_local::um::winnt::__readfsdword;
use std::{
    collections::hash_map::HashMap,
    ffi::{c_schar, OsStr},
    os::windows::prelude::OsStrExt,
};
use winapi::{
    shared::ntdef::{PVOID, ULONG, UNICODE_STRING},
    um::{
        winbase::lstrcmpiW,
        winnt::{
            IMAGE_DIRECTORY_ENTRY_EXPORT, PIMAGE_DOS_HEADER, PIMAGE_EXPORT_DIRECTORY,
            PIMAGE_NT_HEADERS,
        },
    },
};

/// Exports type
pub type Exports = HashMap<String, GenericAddress>;

/// Module view
#[derive(Debug)]
pub struct Module {
    start: usize,
    end: usize,
    /// Holds export address table, gives you virtual address held
    /// in generic addresses container, so you can cast to any type,
    /// or transmute.
    exports: Exports,
}

/// Type for module errors
pub type ModuleErrOr<T> = GenericErrOr<T>;

/// List entry
#[repr(C)]
#[allow(non_camel_case_types)]
#[allow(non_snake_case)]
struct LIST_ENTRY {
    pub Flink: *mut LIST_ENTRY,
    pub Blink: *mut LIST_ENTRY,
}

/// Ldr data entry table for module information
#[repr(C)]
#[allow(non_camel_case_types)]
#[allow(non_snake_case)]
struct LDR_DATA_TABLE_ENTRY {
    pub InMemoryOrderModuleList: LIST_ENTRY,
    pub BaseAddress: PVOID,
    pub EntryPoint: PVOID,
    pub SizeOfImage: ULONG,
    pub FullDllName: UNICODE_STRING,
    pub BaseDllName: UNICODE_STRING,
}

/// Walk process' loaded modules, trying to match for our goal `module`
fn get_module_range(module: &str) -> ModuleErrOr<Module> {
    if !module.ends_with('\0') {
        return Err(Error::NoSentinelCharacter.into());
    }

    // https://users.rust-lang.org/t/how-copy-string-to-c-style-wchar-array/12283

    fn encode_as_wchar(str: &str) -> Vec<u16> {
        OsStr::new(str)
            .encode_wide() // get as wide string
            .chain(Some(0).into_iter()) // add null terminator
            .collect()
    }

    // Get wide-encoded module
    let wide_module = encode_as_wchar(module).as_ptr();

    unsafe {
        // Get process module information
        let peb_ldr_data = *((__readfsdword(0x30) + 0xC) as *const usize);

        // Get first module in initialization order
        let mut it = *((peb_ldr_data + 0x1C) as *const *const LDR_DATA_TABLE_ENTRY);

        loop {
            // Fail condition if not found
            if it.is_null() || ((*it).BaseAddress as usize) == 0 {
                // Iterator is invalid
                break;
            } else if lstrcmpiW((*it).BaseDllName.Buffer, wide_module) == 0 {
                // We have a match
                let start = (*it).BaseAddress as usize;
                let end = start + (*it).SizeOfImage as usize;

                return Ok(Module::new_with(start, end));
            }

            // Advance iteration
            it = (*it).InMemoryOrderModuleList.Flink as _;
        }
    }

    Err(Error::CantFind.into())
}

/// Get `base` module DOS header
fn get_module_dos_header(base: usize) -> PIMAGE_DOS_HEADER {
    base as _
}

/// Get `base` module NT headers
fn get_module_nt_headers(base: usize) -> PIMAGE_NT_HEADERS {
    unsafe { (base + (*get_module_dos_header(base)).e_lfanew as usize) as _ }
}

/// Get `base` module data directory `directory`
fn get_module_data_directory(base: usize, directory: usize) -> usize {
    let nt_headers = get_module_nt_headers(base);
    unsafe { base + (*nt_headers).OptionalHeader.DataDirectory[directory].VirtualAddress as usize }
}

/// Get module exports for module at `base`
fn get_module_exports(base: usize) -> Exports {
    let mut result: Exports = Exports::new();

    unsafe {
        // Get EAT
        let eat = get_module_data_directory(base, IMAGE_DIRECTORY_ENTRY_EXPORT.into())
            as PIMAGE_EXPORT_DIRECTORY;

        // Get tables
        let name_table = (base + (*eat).AddressOfNames as usize) as *const usize;
        let ordinal_table = (base + (*eat).AddressOfNameOrdinals as usize) as *const u16;
        let function_table = (base + (*eat).AddressOfFunctions as usize) as *const usize;

        let number_of_names = (*eat).NumberOfNames;
        for i in 0..number_of_names as isize {
            // Get n-th entry's name
            let raw_name = (base + *name_table.offset(i)) as *mut c_schar;

            // Retrieve C string in Rust
            if let Some(entry_name) = read_c_string(raw_name as _) {
                // Get export address
                let export_address =
                    base + *function_table.offset(*ordinal_table.offset(i) as isize);

                // Verify if there's an offset from base, otherwise it's invalid
                if export_address != base {
                    // Add to hashmap
                    result.insert(entry_name, GenericAddress::from(export_address));
                }
            }
        }
    }

    result
}

impl Module {
    /// Get module with name `module`, store bounds
    ///
    /// # Example
    ///
    /// ```rust
    /// let client = Module::new("client.dll\0").expect("Module not found in process module information");
    /// ```
    pub fn new(module: &str) -> ModuleErrOr<Self> {
        // Calls to new_with
        get_module_range(module)
    }

    /// Get module which you guarantee to be within `start` and `end`
    pub fn new_with(start: usize, end: usize) -> Self {
        Self {
            start,
            end,
            exports: get_module_exports(start),
        }
    }

    /// Get module base address
    pub fn get_start(&self) -> usize {
        self.start
    }

    /// Get module end address
    pub fn get_end(&self) -> usize {
        self.end
    }

    /// Get module size
    pub fn get_size(&self) -> usize {
        self.end - self.start
    }

    /// Get module exports address
    pub fn get_exports(&self) -> &Exports {
        &self.exports
    }

    /// Internal implementation for pattern finding methods
    fn find_nth_pattern_bytes_impl(
        &self,
        pattern: &[u8],
        current_match: usize,
        goal_match: usize,
    ) -> ModuleErrOr<GenericAddress> {
        // Get slice in memory
        let slice = unsafe {
            std::slice::from_raw_parts(self.start as *const u8, self.get_size() - pattern.len())
        };

        /// Needle-in-haystack predicate which acknowledges wildcard
        fn needle_in_haystack(view: &[u8], pattern: &[u8]) -> bool {
            // True if there's no false entry in map (full match)
            !view
                .iter()
                .zip(pattern)
                .map(|(&view_entry, &pattern_entry)| {
                    view_entry == pattern_entry || pattern_entry == 0xCC
                })
                .any(|x| !x)
        }

        // Perform needle-and-haystack on `pattern.len()` windows to view.
        // If succesful, pad iterator by base address.
        // Recurses, need be.
        slice
            .windows(pattern.len())
            .position(|x| needle_in_haystack(x, pattern))
            .ok_or_else(|| Error::CantFind.into())
            .and_then(|x| {
                if current_match != goal_match {
                    // Partially copy current module (without exports)
                    // This would be less of a code smell if modules were called ranges,
                    // But that implies way more overhead in abstraction
                    let next = Self {
                        // `+ 1` makes position not fall on the same window
                        start: self.start + x + 1,
                        end: self.end,
                        exports: Exports::default(),
                    };

                    // Recur till you get to goal
                    next.find_nth_pattern_bytes_impl(pattern, current_match + 1, goal_match)
                } else {
                    Ok(GenericAddress::from(self.start + x))
                }
            })
    }

    /// Find `GOAL`-th instance of `pattern` in `{self.start, self.end}` slice
    ///
    /// # Examples
    ///
    /// ```rust
    /// // Gets second instance of pattern (starts at 0)
    /// let result = Module::new("client.dll\0").find_nth_pattern_bytes(&[0x55u8, 0x8b, 0xec], 1)?;
    /// ```
    pub fn find_nth_pattern_bytes(
        &self,
        pattern: &[u8],
        goal_match: usize,
    ) -> ModuleErrOr<GenericAddress> {
        self.find_nth_pattern_bytes_impl(pattern, 0, goal_match)
    }

    /// Find first instance of `pattern` in `{self.start, self.end}` slice
    ///
    /// # Examples
    ///
    /// ```rust
    /// // Gets second instance of pattern (starts at 0)
    /// let result = Module::new("client.dll\0").find_pattern_bytes(&[0x55u8, 0x8b, 0xec])?;
    /// ```
    pub fn find_pattern_bytes(&self, pattern: &[u8]) -> ModuleErrOr<GenericAddress> {
        self.find_nth_pattern_bytes(pattern, 0)
    }

    /// Find `GOAL`-th instance of `pattern` in `{self.start, self.end}` slice
    ///
    /// # Examples
    ///
    /// ```rust
    /// // Gets second instance of pattern (starts at 0)
    /// let result = Module::new("client.dll\0").find_nth_pattern("55 8b ec", 1)?;
    /// ```
    pub fn find_nth_pattern(
        &self,
        pattern: &str,
        goal_match: usize,
    ) -> ModuleErrOr<GenericAddress> {
        /// Hex pattern string to `Vec<u8>`
        fn string_to_array(pattern: &str) -> Vec<u8> {
            pattern
                .split_whitespace()
                .map(|x| {
                    if x.contains('?') {
                        0xCC
                    } else {
                        u8::from_str_radix(x, 16).unwrap()
                    }
                })
                .collect::<Vec<u8>>()
        }

        self.find_nth_pattern_bytes(string_to_array(pattern).as_slice(), goal_match)
    }

    /// Find first instance of `pattern` in `{self.start, self.end}` slice
    ///
    /// # Examples
    ///
    /// ```rust
    /// // Gets second instance of pattern (starts at 0)
    /// let result = Module::new("client.dll\0").find_pattern("55 8b ec")?;
    /// ```
    pub fn find_pattern(&self, pattern: &str) -> ModuleErrOr<GenericAddress> {
        self.find_nth_pattern(pattern, 0)
    }

    /// Find first instance of C ABI `string` reference in `{self.start, self.end}` slice
    ///
    /// # Examples
    /// ```rust
    /// let reference = Module::new("client.dll\0")?.find_nth_string("CViewRender::SetUpView->OnRenderEnd", 0)?;
    /// ```
    pub fn find_nth_string(&self, string: &str, goal_match: usize) -> ModuleErrOr<GenericAddress> {
        let pattern = string
            .chars()
            .chain(Some('\0')) // add null terminator
            .map(|x| x as u8)
            .collect::<Vec<u8>>();

        // Get address of string in rdata to search for as xref
        let string_address = self.find_pattern_bytes(pattern.as_ref())?;

        // Turn string address to little endianness order bytes and
        // overshadow pattern with it
        let pattern = string_address.exposed_addr().to_le_bytes();

        self.find_nth_pattern_bytes(pattern.as_ref(), goal_match)
    }

    /// Find first instance of C ABI `string` reference in `{self.start, self.end}` slice
    ///
    /// # Examples
    /// ```rust
    /// let reference = Module::new("client.dll\0")?.find_string("CViewRender::SetUpView->OnRenderEnd")?;
    /// ```
    pub fn find_string(&self, string: &str) -> ModuleErrOr<GenericAddress> {
        self.find_nth_string(string, 0)
    }
}
