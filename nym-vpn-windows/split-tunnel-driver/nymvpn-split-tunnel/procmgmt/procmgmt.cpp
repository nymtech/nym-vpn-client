// Copyright 2016-2026 Mullvad VPN AB. All Rights Reserved.
// Copyright 2026 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

#include "procmgmt.h"
#include "context.h"
#include "../util.h"
#include "../defs/config.h"
#include "../defs/events.h"
#include "../eventing/builder.h"

#include "../trace.h"
#include "procmgmt.tmh"

namespace procmgmt {

namespace {

//
// EmitAddEntryErrorEvents()
//
// During process arrival event processing, it may happen that `procregistry::AddEntry()`
// fails to complete successfully.
//
// This has to be communicated through an event.
//
// Additionally, if the arriving process would have been split, then this is a compound
// failure that is communicated separately.
//
void EmitAddEntryErrorEvents(
    eventing::CONTEXT* Eventing,
    NTSTATUS ErrorCode,
    HANDLE ProcessId,
    LOWER_UNICODE_STRING* ImageName,
    bool EmitSplittingEvent) {
    NT_ASSERT(!NT_SUCCESS(ErrorCode));

    DbgPrint("Failed to add entry for arriving process\n");
    DbgPrint("  Status: 0x%X\n", ErrorCode);
    DbgPrint("  PID: %p\n", ProcessId);
    DbgPrint("  Imagename: %wZ\n", (UNICODE_STRING*)ImageName);

    util::StopIfDebugBuild();

    DECLARE_CONST_UNICODE_STRING(errorMessage, L"Failed in call to procregistry::AddEntry()");

    auto errorEvent = eventing::BuildErrorMessageEvent(ErrorCode, &errorMessage);

    eventing::Emit(Eventing, &errorEvent);

    if (!EmitSplittingEvent) {
        return;
    }

    auto splittingErrorEvent = eventing::BuildStartSplittingErrorEvent(ProcessId, ImageName);

    eventing::Emit(Eventing, &splittingErrorEvent);
}

//
// ValidateCollision()
//
// Find and validate existing entry in process registry that prevented the insertion
// of a new entry, because they share the same PID.
//
bool ValidateCollision(CONTEXT* Context, procregistry::PROCESS_REGISTRY_ENTRY const* newEntry) {
    auto processRegistry = Context->ProcessRegistry;

    auto const existingEntry = procregistry::FindEntry(processRegistry->Instance, newEntry->ProcessId);

    if (existingEntry == NULL) {
        DbgPrint("Validate PR collision - could not look up existing entry\n");

        goto Abort_unlock_break;
    }

    if (existingEntry->ParentProcessId != newEntry->ParentProcessId) {
        DbgPrint("Validate PR collision - different parent process\n");

        goto Abort_unlock_break;
    }

    if (existingEntry->ImageName.Length == 0) {
        if (newEntry->ImageName.Length != 0) {
            DbgPrint(
                "Validate PR collision - "
                "registered entry is without image name but proposed entry is not\n");

            goto Abort_unlock_break;
        }

        goto Approved;
    }

    if (!util::Equal(&existingEntry->ImageName, &newEntry->ImageName)) {
        DbgPrint("Validate PR collision - mismatched image name\n");

        goto Abort_unlock_break;
    }

Approved:

    DbgPrint("Process registry collision validation has succeeded\n");

    return true;

Abort_unlock_break:

    DbgPrint("Process registry collision validation has failed\n");

    DbgPrint("Existing entry at %p\n", existingEntry);
    DbgPrint("New proposed entry at %p\n", newEntry);

    util::StopIfDebugBuild();

    return false;
}

struct ArrivalEvent {
    UINT32 SplittingReason;
    bool EmitEvent;

    //
    // Successfully adding a new entry in the process registry makes the
    // registry take ownership of the imagename buffer passed.
    //
    // Therefore, if we need to emit a splitting event for a successful addition,
    // we have to duplicate the imagename here to preserve it.
    //
    LOWER_UNICODE_STRING Imagename;
};

void EvaluateSplitting(CONTEXT* Context, procregistry::PROCESS_REGISTRY_ENTRY* RegistryEntry, ArrivalEvent* ArrivalEvent) {
    auto registeredImage = Context->RegisteredImage->Instance;
    USHORT const flags = registeredimage::GetEntryFlagsExact(registeredImage, &RegistryEntry->ImageName);

    if (flags != 0 || registeredimage::HasEntryExact(registeredImage, &RegistryEntry->ImageName)) {
        bool const isHybrid = (flags & ST_CONFIGURATION_ENTRY_FLAG_HYBRID) != 0;

        if (isHybrid) {
            RegistryEntry->Settings.Split = ST_PROCESS_SPLIT_STATUS_HYBRID_BY_CONFIG;
        } else {
            RegistryEntry->Settings.Split = ST_PROCESS_SPLIT_STATUS_ON_BY_CONFIG;
        }

        ArrivalEvent->SplittingReason |= ST_SPLITTING_REASON_BY_CONFIG;
    } else {
        //
        // Note that we're providing an entry which is not yet added to the registry.
        // This may seem wrong but is totally fine.
        //
        auto processRegistry = Context->ProcessRegistry;

        auto parent = procregistry::GetParentEntry(processRegistry->Instance, RegistryEntry);

        if (parent == NULL || !util::SplittingEnabled(parent->Settings.Split)) {
            return;
        }

        RegistryEntry->Settings.Split = ST_PROCESS_SPLIT_STATUS_ON_BY_INHERITANCE;
        ArrivalEvent->SplittingReason |= ST_SPLITTING_REASON_BY_INHERITANCE;
    }

    ArrivalEvent->EmitEvent = true;

    auto status = util::DuplicateString(&ArrivalEvent->Imagename, &RegistryEntry->ImageName, ST_PAGEABLE::NO);

    if (NT_SUCCESS(status)) {
        return;
    }

    DbgPrint("Cannot emit splitting event for arriving process due to resource exhaustion\n");

    ArrivalEvent->EmitEvent = false;
}

void HandleProcessArriving(CONTEXT* Context, procmon::PROCESS_EVENT const* Record) {
    //
    // State lock is held and is locking out IOCTL handlers.
    //
    // The process registry lock will be required for updating the process registry.
    // The configuration lock won't be required.
    //

    auto processRegistry = Context->ProcessRegistry;

    procregistry::PROCESS_REGISTRY_ENTRY registryEntry = { 0 };

    auto status = procregistry::InitializeEntry(
        processRegistry->Instance,
        Record->Details->ParentProcessId,
        Record->ProcessId,
        ST_PROCESS_SPLIT_STATUS_OFF,
        &(Record->Details->ImageName),
        &registryEntry);

    if (!NT_SUCCESS(status)) {
        DbgPrint("Failed to initialize entry for arriving process\n");
        DbgPrint("  Status: 0x%X\n", status);
        DbgPrint("  PID: %p\n", Record->ProcessId);
        DbgPrint("  Imagename: %wZ\n", &(Record->Details->ImageName));

        return;
    }

    ArrivalEvent arrivalEvent = { .SplittingReason = ST_SPLITTING_REASON_PROCESS_ARRIVING,
                                  .EmitEvent = false,
                                  .Imagename = { 0, 0, NULL } };

    if (Context->EngagedStateActive(Context->CallbackContext)) {
        EvaluateSplitting(Context, &registryEntry, &arrivalEvent);
    }

    //
    // Insert entry into registry.
    //

    WdfSpinLockAcquire(processRegistry->Lock);

    status = procregistry::AddEntry(processRegistry->Instance, &registryEntry);

    WdfSpinLockRelease(processRegistry->Lock);

    if (NT_SUCCESS(status)) {
        //
        // Entry was successfully added and we no longer own the imagename buffer
        // referenced by the registry entry.
        //

        if (arrivalEvent.EmitEvent) {
            auto splittingEvent = eventing::BuildStartSplittingEvent(
                Record->ProcessId,
                (ST_SPLITTING_STATUS_CHANGE_REASON)arrivalEvent.SplittingReason,
                &arrivalEvent.Imagename);

            eventing::Emit(Context->Eventing, &splittingEvent);
        }
    } else if (status == STATUS_DUPLICATE_OBJECTID) {
        //
        // During driver initialization it may happen that the process registry is
        // populated with processes that are also queued to the current function.
        //
        // This is usually fine, but has to be verified to ensure it's an exact duplicate
        // and not just a PID collision.
        //
        // The latter would indicate that events are not being queued in an orderly fashion
        // or went missing alltogether.
        //
        // In case the collision is approved - Do NOT emit an event since the corresponding
        // event will already have been emitted.
        //

        auto const validationStatus = ValidateCollision(Context, &registryEntry);

        if (!validationStatus) {
            EmitAddEntryErrorEvents(
                Context->Eventing, status, registryEntry.ProcessId, &registryEntry.ImageName, arrivalEvent.EmitEvent);
        }

        procregistry::ReleaseEntry(&registryEntry);
    } else {
        //
        // General error handling.
        //

        EmitAddEntryErrorEvents(
            Context->Eventing, status, registryEntry.ProcessId, &registryEntry.ImageName, arrivalEvent.EmitEvent);

        procregistry::ReleaseEntry(&registryEntry);
    }

    //
    // Clean up event data.
    //

    if (arrivalEvent.Imagename.Buffer != NULL) {
        util::FreeStringBuffer(&arrivalEvent.Imagename);
    }

    //
    // No need to update the firewall because the arriving process won't
    // have any existing connections.
    //
}

NTSTATUS
UpdateFirewallDepartingProcess(CONTEXT* Context, procregistry::PROCESS_REGISTRY_ENTRY* registryEntry) {
    //
    // It's inferred that we're in the engaged state.
    // Because we found a process record that has firewall state.
    // But leave this assert here for now.
    //
    NT_ASSERT(Context->EngagedStateActive(Context->CallbackContext));

    auto status = firewall::TransactionBegin(Context->Firewall);

    if (!NT_SUCCESS(status)) {
        DbgPrint("Failed to create firewall transaction: 0x%X\n", status);

        return status;
    }

    status = firewall::RegisterAppBecomingUnsplitTx(Context->Firewall, &registryEntry->ImageName);

    if (!NT_SUCCESS(status)) {
        DbgPrint("Failed to update firewall: 0x%X\n", status);

        auto s2 = firewall::TransactionAbort(Context->Firewall);

        if (!NT_SUCCESS(s2)) {
            DbgPrint("Failed to abort firewall transaction: 0x%X\n", s2);
        }

        return status;
    }

    status = firewall::TransactionCommit(Context->Firewall);

    if (!NT_SUCCESS(status)) {
        DbgPrint("Failed to commit firewall transaction: 0x%X\n", status);

        auto s2 = firewall::TransactionAbort(Context->Firewall);

        if (!NT_SUCCESS(s2)) {
            DbgPrint("Failed to abort firewall transaction: 0x%X\n", s2);
        }
    }

    return status;
}

void HandleProcessDeparting(CONTEXT* Context, procmon::PROCESS_EVENT const* Record) {
    // DbgPrint("Process departing: 0x%X\n", Record->ProcessId);

    //
    // We're still at PASSIVE_LEVEL and the state lock is held.
    // IOCTL handlers are locked out.
    //
    // Complete all processing and acquire the spin lock only when
    // updating the process tree.
    //

    auto processRegistry = Context->ProcessRegistry;

    auto registryEntry = procregistry::FindEntry(processRegistry->Instance, Record->ProcessId);

    if (registryEntry == NULL) {
        DbgPrint("Received process-departing event for unknown PID\n");

        return;
    }

    if (registryEntry->Settings.HasFirewallState) {
        auto status = UpdateFirewallDepartingProcess(Context, registryEntry);

        eventing::RAW_EVENT* evt = NULL;

        if (NT_SUCCESS(status)) {
            evt = eventing::BuildStopSplittingEvent(
                registryEntry->ProcessId, ST_SPLITTING_REASON_PROCESS_DEPARTING, &registryEntry->ImageName);
        } else {
            evt = eventing::BuildStopSplittingErrorEvent(registryEntry->ProcessId, &registryEntry->ImageName);
        }

        eventing::Emit(Context->Eventing, &evt);
    } else if (util::SplittingEnabled(registryEntry->Settings.Split)) {
        auto splittingEvent = eventing::BuildStopSplittingEvent(
            Record->ProcessId, ST_SPLITTING_REASON_PROCESS_DEPARTING, &registryEntry->ImageName);

        eventing::Emit(Context->Eventing, &splittingEvent);
    }

    WdfSpinLockAcquire(processRegistry->Lock);

    bool const deleteSuccessful = procregistry::DeleteEntry(processRegistry->Instance, registryEntry);

    WdfSpinLockRelease(processRegistry->Lock);

    NT_ASSERT(deleteSuccessful);

    //
    // This is unlikely to ever be an issue,
    // but if it was, we'd want to know about it.
    //

    if (!deleteSuccessful) {
        DECLARE_CONST_UNICODE_STRING(errorMessage, L"Failed in call to procregistry::DeleteEntry()");

        auto errorEvent = eventing::BuildErrorMessageEvent(STATUS_UNSUCCESSFUL, &errorMessage);

        eventing::Emit(Context->Eventing, &errorEvent);
    }
}

void NTAPI ProcessEventSink(procmon::PROCESS_EVENT const* Event, void* Context) {
    auto context = (CONTEXT*)Context;

    auto const arriving = (Event->Details != NULL);

    context->AcquireStateLock(context->CallbackContext);

    if (arriving) {
        HandleProcessArriving(context, Event);
    } else {
        HandleProcessDeparting(context, Event);
    }

    context->ReleaseStateLock(context->CallbackContext);

    procbroker::Publish(context->ProcessEventBroker, Event->ProcessId, arriving);
}

} // anonymous namespace

NTSTATUS
Initialize(
    CONTEXT** Context,
    procbroker::CONTEXT* ProcessEventBroker,
    PROCESS_REGISTRY_MGMT* ProcessRegistry,
    REGISTERED_IMAGE_MGMT* RegisteredImage,
    eventing::CONTEXT* Eventing,
    firewall::CONTEXT* Firewall,
    ACQUIRE_STATE_LOCK_FN AcquireStateLock,
    RELEASE_STATE_LOCK_FN ReleaseStateLock,
    ENGAGED_STATE_ACTIVE_FN EngagedStateActive,
    void* CallbackContext) {
    auto context = (CONTEXT*)ExAllocatePoolUninitialized(NonPagedPool, sizeof(CONTEXT), ST_POOL_TAG);

    if (context == NULL) {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    RtlZeroMemory(context, sizeof(*context));

    auto status = procmon::Initialize(&context->ProcessMonitor, ProcessEventSink, context);

    if (!NT_SUCCESS(status)) {
        DbgPrint("procmon::Initialize() failed 0x%X\n", status);

        ExFreePoolWithTag(context, ST_POOL_TAG);

        return status;
    }

    context->ProcessEventBroker = ProcessEventBroker;
    context->ProcessRegistry = ProcessRegistry;
    context->RegisteredImage = RegisteredImage;
    context->Eventing = Eventing;
    context->Firewall = Firewall;

    context->AcquireStateLock = AcquireStateLock;
    context->ReleaseStateLock = ReleaseStateLock;
    context->EngagedStateActive = EngagedStateActive;
    context->CallbackContext = CallbackContext;

    *Context = context;

    return STATUS_SUCCESS;
}

void TearDown(CONTEXT** Context) {
    auto context = *Context;

    procmon::TearDown(&context->ProcessMonitor);

    ExFreePoolWithTag(context, ST_POOL_TAG);

    *Context = NULL;
}

void Activate(CONTEXT* Context) {
    procmon::EnableDispatching(Context->ProcessMonitor);
}

} // namespace procmgmt
