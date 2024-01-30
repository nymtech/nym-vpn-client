#pragma once

#include "memory.h"
#include <string>
#include <windows.h>
#include <aclapi.h>

namespace common::security
{

void AdjustTokenPrivilege(HANDLE token, const std::wstring &privilege, bool enable = true);
void AdjustCurrentProcessTokenPrivilege(const std::wstring &privilege, bool enable = true);

//
// Add built-in group Administrators to DACL with full access.
//
void AddAdminToObjectDacl(const std::wstring &objectName, SE_OBJECT_TYPE objectType);

//
// Create a primary token that represents the security context
// of the specified process.
//
memory::UniqueHandle DuplicateSecurityContext(DWORD processId);

}
