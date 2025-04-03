// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use windows::Win32::{
    Foundation::{GENERIC_ALL, GENERIC_EXECUTE, GENERIC_READ, GENERIC_WRITE},
    Storage::FileSystem::{
        DELETE, FILE_ADD_FILE, FILE_ADD_SUBDIRECTORY, FILE_ALL_ACCESS, FILE_APPEND_DATA,
        FILE_CREATE_PIPE_INSTANCE, FILE_DELETE_CHILD, FILE_EXECUTE, FILE_LIST_DIRECTORY,
        FILE_READ_ATTRIBUTES, FILE_READ_DATA, FILE_READ_EA, FILE_TRAVERSE, FILE_WRITE_ATTRIBUTES,
        FILE_WRITE_DATA, FILE_WRITE_EA, READ_CONTROL, STANDARD_RIGHTS_ALL, STANDARD_RIGHTS_EXECUTE,
        STANDARD_RIGHTS_READ, STANDARD_RIGHTS_REQUIRED, STANDARD_RIGHTS_WRITE, SYNCHRONIZE,
        WRITE_DAC, WRITE_OWNER,
    },
};

bitflags::bitflags! {
    /// File access rights
    ///
    /// Documentation: <https://learn.microsoft.com/en-us/windows/win32/fileio/file-access-rights-constants>
    #[derive(Debug, Clone, Copy)]
    pub struct FileAccessRights: u32 {
        /// tbd
        const FILE_ADD_FILE = FILE_ADD_FILE.0;
        /// tbd
        const FILE_ADD_SUBDIRECTORY = FILE_ADD_SUBDIRECTORY.0;
        /// tbd
        const FILE_ALL_ACCESS = FILE_ALL_ACCESS.0;
        /// tbd
        const FILE_APPEND_DATA = FILE_APPEND_DATA.0;
        /// tbd
        const FILE_CREATE_PIPE_INSTANCE = FILE_CREATE_PIPE_INSTANCE.0;
        /// tbd
        const FILE_DELETE_CHILD = FILE_DELETE_CHILD.0;
        /// tbd
        const FILE_EXECUTE = FILE_EXECUTE.0;
        /// tbd
        const FILE_LIST_DIRECTORY = FILE_LIST_DIRECTORY.0;
        /// tbd
        const FILE_READ_ATTRIBUTES = FILE_READ_ATTRIBUTES.0;
        /// tbd
        const FILE_READ_DATA = FILE_READ_DATA.0;
        /// tbd
        const FILE_READ_EA = FILE_READ_EA.0;
        /// tbd
        const FILE_TRAVERSE = FILE_TRAVERSE.0;
        /// tbd
        const FILE_WRITE_ATTRIBUTES = FILE_WRITE_ATTRIBUTES.0;
        /// tbd
        const FILE_WRITE_DATA = FILE_WRITE_DATA.0;
        /// tbd
        const FILE_WRITE_EA = FILE_WRITE_EA.0;
    }
}

bitflags::bitflags! {
    /// Standard access rights.
    ///
    /// Documentation: <https://learn.microsoft.com/en-us/windows/win32/secauthz/standard-access-rights>
    #[derive(Debug, Clone, Copy)]
    pub struct StandardAccessRights: u32 {
        /// tbd
        const DELETE = DELETE.0;
        /// tbd
        const READ_CONTROL = READ_CONTROL.0;
        /// tbd
        const SYNCHRONIZE = SYNCHRONIZE.0;
        /// tbd
        const WRITE_DAC = WRITE_DAC.0;
        /// tbd
        const WRITE_OWNER = WRITE_OWNER.0;

        /// tbd
        const STANDARD_RIGHTS_ALL = STANDARD_RIGHTS_ALL.0;
        /// tbd
        const STANDARD_RIGHTS_EXECUTE = STANDARD_RIGHTS_EXECUTE.0;
        /// tbd
        const STANDARD_RIGHTS_READ = STANDARD_RIGHTS_READ.0;
        /// tbd
        const STANDARD_RIGHTS_REQUIRED = STANDARD_RIGHTS_REQUIRED.0;
        /// tbd
        const STANDARD_RIGHTS_WRITE = STANDARD_RIGHTS_WRITE.0;
    }
}

bitflags::bitflags! {
    /// Generic access rights.
    ///
    /// Documentation: <https://learn.microsoft.com/en-us/windows/win32/secauthz/generic-access-rights>
    #[derive(Debug, Clone, Copy)]
    pub struct GenericAccessRights: u32 {
        /// tbd
        const GENERIC_READ = GENERIC_READ.0;
        /// tbd
        const GENERIC_WRITE = GENERIC_WRITE.0;
        /// tbd
        const GENERIC_EXECUTE = GENERIC_EXECUTE.0;
        /// tbd
        const GENERIC_ALL = GENERIC_ALL.0;
    }
}
