// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use windows::Win32::{
    Foundation::{GENERIC_ALL, GENERIC_EXECUTE, GENERIC_READ, GENERIC_WRITE},
    Storage::FileSystem::{
        DELETE, FILE_ADD_FILE, FILE_ADD_SUBDIRECTORY, FILE_ALL_ACCESS, FILE_APPEND_DATA,
        FILE_CREATE_PIPE_INSTANCE, FILE_DELETE_CHILD, FILE_EXECUTE, FILE_GENERIC_EXECUTE,
        FILE_GENERIC_READ, FILE_GENERIC_WRITE, FILE_LIST_DIRECTORY, FILE_READ_ATTRIBUTES,
        FILE_READ_DATA, FILE_READ_EA, FILE_TRAVERSE, FILE_WRITE_ATTRIBUTES, FILE_WRITE_DATA,
        FILE_WRITE_EA, READ_CONTROL, STANDARD_RIGHTS_ALL, STANDARD_RIGHTS_EXECUTE,
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
        /// For a directory, the right to create a file in the directory.
        const FILE_ADD_FILE = FILE_ADD_FILE.0;

        /// For a directory, the right to create a subdirectory.
        const FILE_ADD_SUBDIRECTORY = FILE_ADD_SUBDIRECTORY.0;

        /// All possible access rights for a file.
        const FILE_ALL_ACCESS = FILE_ALL_ACCESS.0;

        /// For a file object, the right to append data to the file. (For local files, write operations will not overwrite existing data if this flag is specified without FILE_WRITE_DATA.) For a directory object, the right to create a subdirectory (FILE_ADD_SUBDIRECTORY).
        const FILE_APPEND_DATA = FILE_APPEND_DATA.0;

        /// For a named pipe, the right to create a pipe.
        const FILE_CREATE_PIPE_INSTANCE = FILE_CREATE_PIPE_INSTANCE.0;

        /// For a directory, the right to delete a directory and all the files it contains, including read-only files.
        const FILE_DELETE_CHILD = FILE_DELETE_CHILD.0;

        /// For a native code file, the right to execute the file. This access right given to scripts may cause the script to be executable, depending on the script interpreter.
        const FILE_EXECUTE = FILE_EXECUTE.0;

        /// For a directory, the right to list the contents of the directory.
        const FILE_LIST_DIRECTORY = FILE_LIST_DIRECTORY.0;

        /// The right to read file attributes.
        const FILE_READ_ATTRIBUTES = FILE_READ_ATTRIBUTES.0;

        /// For a file object, the right to read the corresponding file data. For a directory object, the right to read the corresponding directory data.
        const FILE_READ_DATA = FILE_READ_DATA.0;

        /// The right to read extended file attributes.
        const FILE_READ_EXTENDED_ATTRIBUTES = FILE_READ_EA.0;

        /// For a directory, the right to traverse the directory.
        const FILE_TRAVERSE = FILE_TRAVERSE.0;

        /// The right to write file attributes.
        const FILE_WRITE_ATTRIBUTES = FILE_WRITE_ATTRIBUTES.0;

        /// For a file object, the right to write data to the file. For a directory object, the right to create a file in the directory (FILE_ADD_FILE).
        const FILE_WRITE_DATA = FILE_WRITE_DATA.0;

        /// The right to write extended file attributes.
        const FILE_WRITE_EA = FILE_WRITE_EA.0;

        /// Combines FILE_READ_ATTRIBUTES, FILE_READ_DATA, FILE_READ_EA, STANDARD_RIGHTS_READ, SYNCHRONIZE
        const FILE_GENERIC_READ = FILE_GENERIC_READ.0;

        /// Combines FILE_APPEND_DATA, FILE_WRITE_ATTRIBUTES, FILE_WRITE_DATA, FILE_WRITE_EA, STANDARD_RIGHTS_WRITE, SYNCHRONIZE
        const FILE_GENERIC_WRITE = FILE_GENERIC_WRITE.0;

        /// Combines FILE_EXECUTE, FILE_READ_ATTRIBUTES, STANDARD_RIGHTS_EXECUTE, SYNCHRONIZE
        const FILE_GENERIC_EXECUTE = FILE_GENERIC_EXECUTE.0;
    }
}

bitflags::bitflags! {
    /// Standard access rights.
    ///
    /// Documentation: <https://learn.microsoft.com/en-us/windows/win32/secauthz/standard-access-rights>
    #[derive(Debug, Clone, Copy)]
    pub struct StandardAccessRights: u32 {
        /// The right to delete the object.
        const DELETE = DELETE.0;

        /// The right to read the information in the object's security descriptor, not including the information in the system access control list (SACL).
        const READ_CONTROL = READ_CONTROL.0;

        /// The right to use the object for synchronization. This enables a thread to wait until the object is in the signaled state. Some object types do not support this access right.
        const SYNCHRONIZE = SYNCHRONIZE.0;

        /// The right to modify the discretionary access control list (DACL) in the object's security descriptor.
        const WRITE_DAC = WRITE_DAC.0;

        /// The right to change the owner in the object's security descriptor.
        const WRITE_OWNER = WRITE_OWNER.0;

        /// Combines DELETE, READ_CONTROL, WRITE_DAC, WRITE_OWNER, and SYNCHRONIZE access.
        const STANDARD_RIGHTS_ALL = STANDARD_RIGHTS_ALL.0;

        /// Currently defined to equal READ_CONTROL.
        const STANDARD_RIGHTS_EXECUTE = STANDARD_RIGHTS_EXECUTE.0;

        /// Currently defined to equal READ_CONTROL.
        const STANDARD_RIGHTS_READ = STANDARD_RIGHTS_READ.0;

        /// Combines DELETE, READ_CONTROL, WRITE_DAC, and WRITE_OWNER access.
        const STANDARD_RIGHTS_REQUIRED = STANDARD_RIGHTS_REQUIRED.0;

        /// Currently defined to equal READ_CONTROL.
        const STANDARD_RIGHTS_WRITE = STANDARD_RIGHTS_WRITE.0;
    }
}

bitflags::bitflags! {
    /// Generic access rights.
    ///
    /// Documentation: <https://learn.microsoft.com/en-us/windows/win32/secauthz/generic-access-rights>
    #[derive(Debug, Clone, Copy)]
    pub struct GenericAccessRights: u32 {
        /// Read access
        const GENERIC_READ = GENERIC_READ.0;

        /// Write access
        const GENERIC_WRITE = GENERIC_WRITE.0;

        /// Execute access
        const GENERIC_EXECUTE = GENERIC_EXECUTE.0;

        /// All possible access rights
        const GENERIC_ALL = GENERIC_ALL.0;
    }
}

impl From<FileAccessRights> for AccessRights {
    fn from(rights: FileAccessRights) -> Self {
        Self {
            file_access_rights: Some(rights),
            ..Default::default()
        }
    }
}

impl From<GenericAccessRights> for AccessRights {
    fn from(rights: GenericAccessRights) -> Self {
        Self {
            generic_access_rights: Some(rights),
            ..Default::default()
        }
    }
}

impl From<StandardAccessRights> for AccessRights {
    fn from(rights: StandardAccessRights) -> Self {
        Self {
            standard_access_rights: Some(rights),
            ..Default::default()
        }
    }
}

/// Struct holding various access rights that can be combined together.
#[derive(Debug, Clone, Copy, Default)]
pub struct AccessRights {
    file_access_rights: Option<FileAccessRights>,
    standard_access_rights: Option<StandardAccessRights>,
    generic_access_rights: Option<GenericAccessRights>,
}

impl AccessRights {
    pub fn set_file_access_rights(mut self, rights: FileAccessRights) -> Self {
        self.file_access_rights = Some(rights);
        self
    }

    pub fn set_standard_access_rights(mut self, rights: StandardAccessRights) -> Self {
        self.standard_access_rights = Some(rights);
        self
    }

    pub fn set_generic_access_rights(mut self, rights: GenericAccessRights) -> Self {
        self.generic_access_rights = Some(rights);
        self
    }

    /// Combine all access rights into a single bitmask.
    pub fn bits(&self) -> u32 {
        let mut permissions: u32 = 0;

        if let Some(file_access_rights) = self.file_access_rights {
            permissions |= file_access_rights.bits();
        }

        if let Some(standard_access_rights) = self.standard_access_rights {
            permissions |= standard_access_rights.bits();
        }

        if let Some(generic_access_rights) = self.generic_access_rights {
            permissions |= generic_access_rights.bits();
        }

        permissions
    }
}
