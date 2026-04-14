// Copyright 2016-2026 Mullvad VPN AB. All Rights Reserved.
// Copyright 2026 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

#include <ntifs.h>
#include "util.h"

namespace util {

void ReparentList(LIST_ENTRY* Dest, LIST_ENTRY* Src) {
    //
    // If it's an empty list there is nothing to reparent.
    //

    if (Src->Flink == Src) {
        InitializeListHead(Dest);
        return;
    }

    //
    // Replace root node.
    //

    *Dest = *Src;

    //
    // Update links on first and last entry.
    //

    Dest->Flink->Blink = Dest;
    Dest->Blink->Flink = Dest;

    //
    // Reinitialize original root node.
    //

    InitializeListHead(Src);
}

typedef NTSTATUS (*QUERY_INFO_PROCESS)(
    __in HANDLE ProcessHandle,
    __in PROCESSINFOCLASS ProcessInformationClass,
    __out_bcount(ProcessInformationLength) PVOID ProcessInformation,
    __in ULONG ProcessInformationLength,
    __out_opt PULONG ReturnLength);

extern "C" NTSTATUS GetDevicePathImageName(PEPROCESS Process, UNICODE_STRING** ImageName) {
    *ImageName = NULL;

    HANDLE processHandle;

    auto status =
        ObOpenObjectByPointer(Process, OBJ_KERNEL_HANDLE, NULL, GENERIC_READ, NULL, KernelMode, &processHandle);

    if (!NT_SUCCESS(status)) {
        return status;
    }

    static QUERY_INFO_PROCESS QueryFunction = NULL;

    if (QueryFunction == NULL) {
        DECLARE_CONST_UNICODE_STRING(queryName, L"ZwQueryInformationProcess");

        QueryFunction = (QUERY_INFO_PROCESS)MmGetSystemRoutineAddress((UNICODE_STRING*)&queryName);

        if (NULL == QueryFunction) {
            // TODO: Use more appropriate error code
            status = STATUS_NOT_CAPABLE;

            goto Failure;
        }
    }

    //
    // Determine required size of name buffer.
    //

    ULONG bufferLength;

    status = QueryFunction(processHandle, ProcessImageFileName, NULL, 0, &bufferLength);

    if (status != STATUS_INFO_LENGTH_MISMATCH) {
        goto Failure;
    }

    //
    // Allocate name buffer.
    //

    *ImageName = (UNICODE_STRING*)ExAllocatePoolUninitialized(PagedPool, bufferLength, ST_POOL_TAG);

    if (NULL == *ImageName) {
        status = STATUS_INSUFFICIENT_RESOURCES;

        goto Failure;
    }

    //
    // Retrieve filename.
    //

    status = QueryFunction(processHandle, ProcessImageFileName, *ImageName, bufferLength, &bufferLength);

    if (NT_SUCCESS(status)) {
        goto Cleanup;
    }

Failure:

    if (*ImageName != NULL) {
        ExFreePoolWithTag(*ImageName, ST_POOL_TAG);
    }

Cleanup:

    ZwClose(processHandle);

    return status;
}

bool ValidateBufferRange(void const* Buffer, void const* BufferEnd, SIZE_T RangeOffset, SIZE_T RangeLength) {
    if (RangeLength == 0) {
        return true;
    }

    auto range = (const UCHAR*)Buffer + RangeOffset;
    auto rangeEnd = range + RangeLength;

    if (range < (const UCHAR*)Buffer || range >= (const UCHAR*)BufferEnd || rangeEnd < range || rangeEnd > BufferEnd) {
        return false;
    }

    return true;
}

bool IsEmptyRange(void const* Buffer, SIZE_T Length) {
    //
    // TODO
    //
    // Assuming x64, round down `Length` and read QWORDs from the buffer.
    // Then read the last few bytes in this silly byte-by-byte manner.
    //

    for (auto b = (const UCHAR*)Buffer; Length != 0; ++b, --Length) {
        if (*b != 0) {
            return false;
        }
    }

    return true;
}

NTSTATUS
AllocateCopyDowncaseString(LOWER_UNICODE_STRING* Dest, const UNICODE_STRING* const Src, ST_PAGEABLE Pageable) {
    //
    // Unfortunately, there is no way to determine the required buffer size.
    //
    // It would be possible to allocate e.g. `In.Length * 1.5` bytes, and waste memory.
    //
    // We opt for the slightly less time efficient method of allocating an exact size
    // twice and copying the string.
    //

    UNICODE_STRING lower;

    auto status = RtlDowncaseUnicodeString(&lower, Src, TRUE);

    if (!NT_SUCCESS(status)) {
        return status;
    }

    auto const poolType = (Pageable == ST_PAGEABLE::YES) ? PagedPool : NonPagedPool;

    auto finalBuffer = (PWCH)ExAllocatePoolUninitialized(poolType, lower.Length, ST_POOL_TAG);

    if (finalBuffer == NULL) {
        RtlFreeUnicodeString(&lower);

        return STATUS_INSUFFICIENT_RESOURCES;
    }

    RtlCopyMemory(finalBuffer, lower.Buffer, lower.Length);

    Dest->Length = lower.Length;
    Dest->MaximumLength = lower.Length;
    Dest->Buffer = finalBuffer;

    RtlFreeUnicodeString(&lower);

    return STATUS_SUCCESS;
}

void FreeStringBuffer(UNICODE_STRING* String) {
    ExFreePoolWithTag(String->Buffer, ST_POOL_TAG);

    String->Length = 0;
    String->MaximumLength = 0;
    String->Buffer = NULL;
}

void FreeStringBuffer(LOWER_UNICODE_STRING* String) {
    return FreeStringBuffer((UNICODE_STRING*)String);
}

NTSTATUS
DuplicateString(UNICODE_STRING* Dest, const UNICODE_STRING* Src, ST_PAGEABLE Pageable) {
    auto const poolType = (Pageable == ST_PAGEABLE::YES) ? PagedPool : NonPagedPool;

    auto buffer = (PWCH)ExAllocatePoolUninitialized(poolType, Src->Length, ST_POOL_TAG);

    if (buffer == NULL) {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    RtlCopyMemory(buffer, Src->Buffer, Src->Length);

    Dest->Length = Src->Length;
    Dest->MaximumLength = Src->Length;
    Dest->Buffer = buffer;

    return STATUS_SUCCESS;
}

NTSTATUS
DuplicateString(LOWER_UNICODE_STRING* Dest, const LOWER_UNICODE_STRING* Src, ST_PAGEABLE Pageable) {
    return DuplicateString((UNICODE_STRING*)Dest, (const UNICODE_STRING*)Src, Pageable);
}

void StopIfDebugBuild() {
#ifdef DEBUG
    DbgBreakPoint();
#endif
}

bool SplittingEnabled(ST_PROCESS_SPLIT_STATUS Status) {
    return (Status == ST_PROCESS_SPLIT_STATUS_ON_BY_CONFIG || Status == ST_PROCESS_SPLIT_STATUS_ON_BY_INHERITANCE);
}

bool HybridEnabled(ST_PROCESS_SPLIT_STATUS Status) {
    return Status == ST_PROCESS_SPLIT_STATUS_HYBRID_BY_CONFIG;
}

bool Equal(const LOWER_UNICODE_STRING* lhs, const LOWER_UNICODE_STRING* rhs) {
    if (lhs->Length != rhs->Length) {
        return false;
    }

    auto const equalBytes = RtlCompareMemory(lhs->Buffer, rhs->Buffer, lhs->Length);

    return equalBytes == lhs->Length;
}

void Swap(LOWER_UNICODE_STRING* lhs, LOWER_UNICODE_STRING* rhs) {
    const LOWER_UNICODE_STRING temp = *lhs;

    *lhs = *rhs;

    *rhs = temp;
}

} // namespace util
