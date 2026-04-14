// Copyright 2016-2026 Mullvad VPN AB. All Rights Reserved.
// Copyright 2026 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

#include "ioctl.h"
#include "devicecontext.h"
#include "util.h"
#include "ipaddr.h"
#include "firewall/firewall.h"
#include "defs/config.h"
#include "defs/process.h"
#include "defs/queryprocess.h"
#include "validation.h"
#include "eventing/eventing.h"
#include "eventing/builder.h"

#include "trace.h"
#include "ioctl.tmh"

namespace ioctl {

namespace {

//
// Minimum buffer sizes for requests.
//
enum class MIN_REQUEST_SIZE {
    SET_CONFIGURATION = sizeof(ST_CONFIGURATION_HEADER),
    GET_CONFIGURATION = sizeof(SIZE_T),
    REGISTER_PROCESSES = sizeof(ST_PROCESS_DISCOVERY_HEADER),
    REGISTER_IP_ADDRESSES = sizeof(ST_IP_ADDRESSES),
    GET_IP_ADDRESSES = sizeof(ST_IP_ADDRESSES),
    GET_STATE = sizeof(SIZE_T),
    QUERY_PROCESS = sizeof(ST_QUERY_PROCESS),
    QUERY_PROCESS_RESPONSE = sizeof(ST_QUERY_PROCESS_RESPONSE),
};

bool VpnActive(const ST_IP_ADDRESSES* IpAddresses) {
    return ip::ValidTunnelIpv4Address(IpAddresses) || ip::ValidTunnelIpv6Address(IpAddresses);
}

NTSTATUS
InitializeProcessRegistryMgmt(PROCESS_REGISTRY_MGMT* Mgmt) {
    auto status = WdfSpinLockCreate(WDF_NO_OBJECT_ATTRIBUTES, &Mgmt->Lock);

    if (!NT_SUCCESS(status)) {
        DbgPrint("WdfSpinLockCreate() failed 0x%X\n", status);

        goto Abort;
    }

    status = procregistry::Initialize(&Mgmt->Instance, ST_PAGEABLE::NO);

    if (!NT_SUCCESS(status)) {
        DbgPrint("procregistry::Initialize() failed 0x%X\n", status);

        goto Abort_Delete_Lock;
    }

    return STATUS_SUCCESS;

Abort_Delete_Lock:

    WdfObjectDelete(Mgmt->Lock);

Abort:

    Mgmt->Lock = NULL;
    Mgmt->Instance = NULL;

    return status;
}

void DestroyProcessRegistryMgmt(PROCESS_REGISTRY_MGMT* Mgmt) {
    if (Mgmt->Instance != NULL) {
        procregistry::TearDown(&Mgmt->Instance);
        Mgmt->Instance = NULL;
    }

    if (Mgmt->Lock != NULL) {
        WdfObjectDelete(Mgmt->Lock);
        Mgmt->Lock = NULL;
    }
}

//
// UpdateTargetSplitSetting()
//
// Updates the target split setting on a process registry entry.
//
// Target state is set to split if either of:
//
// - Imagename is included in config.
// - Currently split by inheritance and parent has departed.
//
bool NTAPI UpdateTargetSplitSetting(procregistry::PROCESS_REGISTRY_ENTRY* Entry, void* Context) {
    auto context = (ST_DEVICE_CONTEXT*)Context;

    Entry->TargetSettings.Split = ST_PROCESS_SPLIT_STATUS_OFF;

    USHORT const flags = registeredimage::GetEntryFlagsExact(context->RegisteredImage.Instance, &Entry->ImageName);

    if (flags != 0 || registeredimage::HasEntryExact(context->RegisteredImage.Instance, &Entry->ImageName)) {
        bool const isHybrid = (flags & ST_CONFIGURATION_ENTRY_FLAG_HYBRID) != 0;

        Entry->TargetSettings.Split = isHybrid ? ST_PROCESS_SPLIT_STATUS_HYBRID_BY_CONFIG
                                               : ST_PROCESS_SPLIT_STATUS_ON_BY_CONFIG;
    } else if (Entry->ParentProcessId == 0 && Entry->Settings.Split == ST_PROCESS_SPLIT_STATUS_ON_BY_INHERITANCE) {
        Entry->TargetSettings.Split = ST_PROCESS_SPLIT_STATUS_ON_BY_INHERITANCE;
    }

    return true;
}

//
// ApplyFinalizeTargetSettings()
//
// NOTE: Applies the target split setting but does not update current settings
// on the process registry entry under consideration.
//
// Manages transitions in settings changes:
//
// Not split -> split
// Split -> not split
//
// Something worth noting is that a process being split may have firewall state, but a process
// that's not being split will never have firewall state.
//
// This is contrary to a previous design that used additional filters to block non-tunnel traffic.
//
bool NTAPI ApplyFinalizeTargetSettings(ST_DEVICE_CONTEXT* Context, procregistry::PROCESS_REGISTRY_ENTRY* Entry) {
    if (!util::SplittingEnabled(Entry->Settings.Split) && !util::HybridEnabled(Entry->Settings.Split)) {
        NT_ASSERT(!Entry->Settings.HasFirewallState);

        if (!util::SplittingEnabled(Entry->TargetSettings.Split)) {
            Entry->TargetSettings.HasFirewallState = false;

            return true;
        }

        //
        // Not split -> split
        //

        auto status = firewall::RegisterAppBecomingSplitTx(Context->Firewall, &Entry->ImageName);

        if (!NT_SUCCESS(status)) {
            return false;
        }

        return Entry->TargetSettings.HasFirewallState = true;
    }

    if (util::HybridEnabled(Entry->Settings.Split)) {
        NT_ASSERT(!Entry->Settings.HasFirewallState);

        if (util::SplittingEnabled(Entry->TargetSettings.Split)) {
            //
            // Hybrid -> split: register tunnel-blocking filter for this app.
            //

            auto status = firewall::RegisterAppBecomingSplitTx(Context->Firewall, &Entry->ImageName);

            if (!NT_SUCCESS(status)) {
                return false;
            }

            return Entry->TargetSettings.HasFirewallState = true;
        }

        //
        // Hybrid -> not split or hybrid -> hybrid: no firewall state needed.
        //

        Entry->TargetSettings.HasFirewallState = false;

        return true;
    }

    if (util::SplittingEnabled(Entry->TargetSettings.Split)) {
        Entry->TargetSettings.HasFirewallState = Entry->Settings.HasFirewallState;

        return true;
    }

    //
    // Split -> not split or split -> hybrid
    //

    if (Entry->Settings.HasFirewallState) {
        auto status = firewall::RegisterAppBecomingUnsplitTx(Context->Firewall, &Entry->ImageName);

        if (!NT_SUCCESS(status)) {
            return false;
        }
    }

    Entry->TargetSettings.HasFirewallState = false;

    return true;
}

//
// PropagateApplyTargetSettings()
//
// Traverse ancestry to see if parent/grandparent/etc is being split.
// Then apply target split setting.
//
bool NTAPI PropagateApplyTargetSettings(procregistry::PROCESS_REGISTRY_ENTRY* Entry, void* Context) {
    auto context = (ST_DEVICE_CONTEXT*)Context;

    if (!util::SplittingEnabled(Entry->TargetSettings.Split)) {
        auto currentEntry = Entry;

        //
        // In the current state of changing settings,
        // we have to follow the ancestry all the way to the root.
        //

        for (;;) {
            auto const parent = procregistry::GetParentEntry(context->ProcessRegistry.Instance, currentEntry);

            if (parent == NULL) {
                break;
            }

            if (util::SplittingEnabled(parent->TargetSettings.Split)) {
                Entry->TargetSettings.Split = ST_PROCESS_SPLIT_STATUS_ON_BY_INHERITANCE;
                break;
            }

            currentEntry = parent;
        }
    }

    return ApplyFinalizeTargetSettings(context, Entry);
}

struct CONFIGURATION_COMPUTE_LENGTH_CONTEXT {
    SIZE_T NumEntries;
    SIZE_T TotalStringLength;
};

bool NTAPI GetConfigurationComputeLength(const LOWER_UNICODE_STRING* Entry, USHORT Flags, void* Context) {
    UNREFERENCED_PARAMETER(Flags);
    auto ctx = (CONFIGURATION_COMPUTE_LENGTH_CONTEXT*)Context;

    ++(ctx->NumEntries);

    ctx->TotalStringLength += Entry->Length;

    return true;
}

struct CONFIGURATION_SERIALIZE_CONTEXT {
    // Next entry that should be written.
    ST_CONFIGURATION_ENTRY* Entry;

    // Pointer where next string should be written.
    UCHAR* StringDest;

    // Offset where next string should be written.
    SIZE_T StringOffset;
};

bool NTAPI GetConfigurationSerialize(const LOWER_UNICODE_STRING* Entry, USHORT Flags, void* Context) {
    auto ctx = (CONFIGURATION_SERIALIZE_CONTEXT*)Context;

    //
    // Copy data.
    //

    ctx->Entry->ImageNameOffset = ctx->StringOffset;
    ctx->Entry->ImageNameLength = Entry->Length;
    ctx->Entry->Flags = Flags;

    RtlCopyMemory(ctx->StringDest, Entry->Buffer, Entry->Length);

    //
    // Update context for next iteration.
    //

    ++(ctx->Entry);
    ctx->StringDest += Entry->Length;
    ctx->StringOffset += Entry->Length;

    return true;
}

//
// CallbackQueryProcess
//
// This callback is provided to the firewall for use with callouts.
//
// We don't need to worry about the current driver state, because if callouts
// are active this means the current state is "engaged".
//
firewall::PROCESS_SPLIT_VERDICT CallbackQueryProcess(HANDLE ProcessId, void* RawContext) {
    auto context = (ST_DEVICE_CONTEXT*)RawContext;

    WdfSpinLockAcquire(context->ProcessRegistry.Lock);

    auto process = procregistry::FindEntry(context->ProcessRegistry.Instance, ProcessId);

    firewall::PROCESS_SPLIT_VERDICT verdict = firewall::PROCESS_SPLIT_VERDICT::UNKNOWN;

    if (process != NULL) {
        if (util::SplittingEnabled(process->Settings.Split)) {
            verdict = firewall::PROCESS_SPLIT_VERDICT::DO_SPLIT;
        } else if (util::HybridEnabled(process->Settings.Split)) {
            verdict = firewall::PROCESS_SPLIT_VERDICT::BYPASS;
        } else {
            verdict = firewall::PROCESS_SPLIT_VERDICT::DONT_SPLIT;
        }
    }

    WdfSpinLockRelease(context->ProcessRegistry.Lock);

    return verdict;
}

bool NTAPI DbgPrintConfiguration(const LOWER_UNICODE_STRING* Entry, USHORT Flags, void* Context) {
    UNREFERENCED_PARAMETER(Flags);
    UNREFERENCED_PARAMETER(Context);

    DbgPrint("%wZ\n", (const UNICODE_STRING*)Entry);

    return true;
}

//
// RealizeAnnounceSettingsChange()
//
// Update previous, current settings.
//
// Analyze change and emit corresponding event.
//
bool NTAPI RealizeAnnounceSettingsChange(procregistry::PROCESS_REGISTRY_ENTRY* Entry, void* Context) {
    auto context = (ST_DEVICE_CONTEXT*)Context;

    Entry->PreviousSettings = Entry->Settings;
    Entry->Settings = Entry->TargetSettings;

    if (util::SplittingEnabled(Entry->Settings.Split)) {
        if (!util::SplittingEnabled(Entry->PreviousSettings.Split)) {
            auto evt =
                eventing::BuildStartSplittingEvent(Entry->ProcessId, ST_SPLITTING_REASON_BY_CONFIG, &Entry->ImageName);

            eventing::Emit(context->Eventing, &evt);
        }
    } else {
        if (util::SplittingEnabled(Entry->PreviousSettings.Split)) {
            auto evt =
                eventing::BuildStopSplittingEvent(Entry->ProcessId, ST_SPLITTING_REASON_BY_CONFIG, &Entry->ImageName);

            eventing::Emit(context->Eventing, &evt);
        }
    }

    return true;
}

//
// ClearRealizeAnnounceSettingsChange()
//
// Clear splitting. Then realize and announce.
//
bool NTAPI ClearRealizeAnnounceSettingsChange(procregistry::PROCESS_REGISTRY_ENTRY* Entry, void* Context) {
    Entry->TargetSettings.Split = ST_PROCESS_SPLIT_STATUS_OFF;
    Entry->TargetSettings.HasFirewallState = false;

    return RealizeAnnounceSettingsChange(Entry, Context);
}

NTSTATUS
SyncProcessRegistry(ST_DEVICE_CONTEXT* Context, bool ForceAleReauthorization = false) {
    //
    // The process management subsystem is locked out becase we're holding the state lock.
    // This ensures there will be no structural changes to the process registry.
    //
    // There will be readers at DISPATCH (callouts).
    // But we are free to make atomic updates to individual entries.
    //
    // Locking of the configuration is not required since we're in the serialized
    // IOCTL handler path.
    //

    procregistry::ForEach(Context->ProcessRegistry.Instance, UpdateTargetSplitSetting, Context);

    auto status = firewall::TransactionBegin(Context->Firewall);

    if (!NT_SUCCESS(status)) {
        DbgPrint("Could not create firewall transaction: 0x%X\n", status);

        return status;
    }

    auto successful = procregistry::ForEach(Context->ProcessRegistry.Instance, PropagateApplyTargetSettings, Context);

    if (!successful) {
        DbgPrint("Could not add/remove firewall filters\n");

        status = STATUS_UNSUCCESSFUL;

        goto Abort;
    }

    status = firewall::TransactionCommit(Context->Firewall, ForceAleReauthorization);

    if (!NT_SUCCESS(status)) {
        DbgPrint("Could not commit firewall transaction\n");

        goto Abort;
    }

    //
    // No fallible operations beyond here.
    //
    // Send splitting events and finish off.
    //

    procregistry::ForEach(Context->ProcessRegistry.Instance, RealizeAnnounceSettingsChange, Context);

    return STATUS_SUCCESS;

Abort:

    auto s2 = firewall::TransactionAbort(Context->Firewall);

    if (!NT_SUCCESS(s2)) {
        DbgPrint("Could not abort firewall transaction: 0x%X\n", s2);
    }

    return status;
}

NTSTATUS
EnterEngagedState(ST_DEVICE_CONTEXT* Context, const ST_IP_ADDRESSES* IpAddresses) {
    auto status = firewall::EnableSplitting(Context->Firewall, IpAddresses);

    if (!NT_SUCCESS(status)) {
        DbgPrint("Could not enable splitting in firewall: 0x%X\n", status);

        return status;
    }

    status = SyncProcessRegistry(Context);

    if (!NT_SUCCESS(status)) {
        DbgPrint("Could not synchronize process registry with configuration: 0x%X\n", status);

        auto s2 = firewall::DisableSplitting(Context->Firewall);

        if (!NT_SUCCESS(s2)) {
            DbgPrint("DisableSplitting() failed: 0x%X\n", s2);
        }

        return status;
    }

    Context->DriverState.State = ST_DRIVER_STATE_ENGAGED;

    DbgPrint("Successful state transition READY -> ENGAGED\n");

    return STATUS_SUCCESS;
}

NTSTATUS
LeaveEngagedState(ST_DEVICE_CONTEXT* Context) {
    auto status = firewall::DisableSplitting(Context->Firewall);

    if (!NT_SUCCESS(status)) {
        DbgPrint("Could not disable splitting in firewall: 0x%X\n", status);

        return status;
    }

    //
    // This doesn't touch the firewall.
    // It's already been reset as a result of the disable-call above.
    //
    procregistry::ForEach(Context->ProcessRegistry.Instance, ClearRealizeAnnounceSettingsChange, Context);

    Context->DriverState.State = ST_DRIVER_STATE_READY;

    DbgPrint("Successful state transition ENGAGED -> READY\n");

    return STATUS_SUCCESS;
}

NTSTATUS
RegisterIpAddressesAtReady(ST_DEVICE_CONTEXT* Context, const ST_IP_ADDRESSES* newIpAddresses) {
    //
    // If there's no config registered we just store the addresses and succeed.
    //
    // No need to access the configuration exclusively:
    //
    // - We're in the serialized IOCTL handler path.
    // - Config is only read from, not written to.
    //

    if (registeredimage::IsEmpty(Context->RegisteredImage.Instance)) {
        Context->IpAddresses = *newIpAddresses;

        return STATUS_SUCCESS;
    }

    //
    // There's a configuration registered.
    //
    // However, if the VPN isn't active we can't enter the engaged state.
    //

    if (!VpnActive(newIpAddresses)) {
        Context->IpAddresses = *newIpAddresses;

        return STATUS_SUCCESS;
    }

    //
    // Enter into engaged state.
    //

    auto status = EnterEngagedState(Context, newIpAddresses);

    if (!NT_SUCCESS(status)) {
        DbgPrint("Could not enter engaged state: 0x%X\n", status);

        return status;
    }

    Context->IpAddresses = *newIpAddresses;

    return STATUS_SUCCESS;
}

NTSTATUS
RegisterIpAddressesAtEngaged(ST_DEVICE_CONTEXT* Context, const ST_IP_ADDRESSES* newIpAddresses) {
    if (!VpnActive(newIpAddresses)) {
        auto status = LeaveEngagedState(Context);

        if (!NT_SUCCESS(status)) {
            DbgPrint("Could not leave engaged state: 0x%X\n", status);

            return status;
        }

        Context->IpAddresses = *newIpAddresses;

        return STATUS_SUCCESS;
    }

    //
    // No state change required.
    // Notify firewall so it can rewrite any filters with IP-conditions.
    //

    auto status = firewall::RegisterUpdatedIpAddresses(Context->Firewall, newIpAddresses);

    if (!NT_SUCCESS(status)) {
        DbgPrint("Could not update firewall with new IPs: 0x%X\n", status);

        return status;
    }

    Context->IpAddresses = *newIpAddresses;

    return STATUS_SUCCESS;
}

NTSTATUS
RegisterConfigurationAtReady(ST_DEVICE_CONTEXT* Context, registeredimage::CONTEXT* Imageset) {
    //
    // If VPN is not active just store new configuration and succeed.
    //

    if (!VpnActive(&Context->IpAddresses)) {
        auto oldConfiguration = Context->RegisteredImage.Instance;

        Context->RegisteredImage.Instance = Imageset;

        registeredimage::TearDown(&oldConfiguration);

        return STATUS_SUCCESS;
    }

    //
    // VPN is active so enter engaged state.
    //

    auto oldConfiguration = Context->RegisteredImage.Instance;

    Context->RegisteredImage.Instance = Imageset;

    auto status = EnterEngagedState(Context, &Context->IpAddresses);

    if (!NT_SUCCESS(status)) {
        DbgPrint("Could not enter engaged state: 0x%X\n", status);

        Context->RegisteredImage.Instance = oldConfiguration;

        registeredimage::TearDown(&Imageset);

        return status;
    }

    registeredimage::TearDown(&oldConfiguration);

    return STATUS_SUCCESS;
}

NTSTATUS
RegisterConfigurationAtEngaged(ST_DEVICE_CONTEXT* Context, registeredimage::CONTEXT* Imageset) {
    auto oldConfiguration = Context->RegisteredImage.Instance;

    Context->RegisteredImage.Instance = Imageset;

    //
    // Update process registry to reflect new configuration.
    //

    auto status = SyncProcessRegistry(Context, true);

    if (!NT_SUCCESS(status)) {
        DbgPrint("Could not synchronize process registry with configuration: 0x%X\n", status);

        Context->RegisteredImage.Instance = oldConfiguration;

        registeredimage::TearDown(&Imageset);

        return status;
    }

    registeredimage::TearDown(&oldConfiguration);

    return STATUS_SUCCESS;
}

void NTAPI CallbackAcquireStateLock(void* Context) {
    auto context = (ST_DEVICE_CONTEXT*)Context;

    WdfWaitLockAcquire(context->DriverState.Lock, NULL);
}

void NTAPI CallbackReleaseStateLock(void* Context) {
    auto context = (ST_DEVICE_CONTEXT*)Context;

    WdfWaitLockRelease(context->DriverState.Lock);
}

bool NTAPI CallbackEngagedStateActive(void* Context) {
    auto context = (ST_DEVICE_CONTEXT*)Context;

    return context->DriverState.State == ST_DRIVER_STATE_ENGAGED;
}

NTSTATUS
ResetInner(ST_DEVICE_CONTEXT* Context) {
    //
    // Leave engaged state to minimize the impact if any of this fails.
    //

    if (Context->DriverState.State == ST_DRIVER_STATE_ENGAGED) {
        WdfWaitLockAcquire(Context->DriverState.Lock, NULL);

        auto status = LeaveEngagedState(Context);

        WdfWaitLockRelease(Context->DriverState.Lock);

        if (!NT_SUCCESS(status)) {
            DbgPrint("Could not leave engaged state\n");
        }
    }

    //
    // Tear down everything in reverse order of initializing it.
    //

    procmgmt::TearDown(&Context->ProcessMgmt);

    auto status = firewall::TearDown(&Context->Firewall);

    if (!NT_SUCCESS(status)) {
        //
        // Filters or callouts could not be unregistered.
        //
        // There is no way to recover from this. The driver will not be able to unload.
        //
        // All moving parts in the system that depend on the state lock are stopped.
        // So safe to update state without using the lock.
        //

        Context->DriverState.State = ST_DRIVER_STATE_ZOMBIE;

        return status;
    }

    RtlZeroMemory(&Context->IpAddresses, sizeof(Context->IpAddresses));

    procregistry::TearDown(&Context->ProcessRegistry.Instance);

    registeredimage::TearDown((registeredimage::CONTEXT**)&Context->RegisteredImage.Instance);

    procbroker::TearDown(&Context->ProcessEventBroker);

    eventing::TearDown(&Context->Eventing);

    Context->DriverState.State = ST_DRIVER_STATE_STARTED;

    return STATUS_SUCCESS;
}

} // anonymous namespace

NTSTATUS
Initialize(WDFDEVICE Device) {
    auto context = DeviceGetSplitTunnelContext(Device);

    //
    // The context struct is cleared.
    // Only state is set at this point.
    //

    auto status = eventing::Initialize(&context->Eventing, Device);

    if (!NT_SUCCESS(status)) {
        return status;
    }

    status = procbroker::Initialize(&context->ProcessEventBroker);

    if (!NT_SUCCESS(status)) {
        goto Abort_teardown_eventing;
    }

    status =
        registeredimage::Initialize((registeredimage::CONTEXT**)&context->RegisteredImage.Instance, ST_PAGEABLE::NO);

    if (!NT_SUCCESS(status)) {
        goto Abort_teardown_procbroker;
    }

    status = InitializeProcessRegistryMgmt(&context->ProcessRegistry);

    if (!NT_SUCCESS(status)) {
        goto Abort_teardown_registeredimage;
    }

    firewall::CALLBACKS callbacks;

    callbacks.QueryProcess = CallbackQueryProcess;
    callbacks.Context = context;

    status = firewall::Initialize(
        &context->Firewall,
        WdfDeviceWdmGetDeviceObject(Device),
        &callbacks,
        context->ProcessEventBroker,
        context->Eventing);

    if (!NT_SUCCESS(status)) {
        goto Abort_teardown_process_registry;
    }

    status = procmgmt::Initialize(
        &context->ProcessMgmt,
        context->ProcessEventBroker,
        &context->ProcessRegistry,
        &context->RegisteredImage,
        context->Eventing,
        context->Firewall,
        CallbackAcquireStateLock,
        CallbackReleaseStateLock,
        CallbackEngagedStateActive,
        context);

    if (!NT_SUCCESS(status)) {
        goto Abort_teardown_firewall;
    }

    context->DriverState.State = ST_DRIVER_STATE_INITIALIZED;

    DbgPrint("Successfully processed IOCTL_ST_INITIALIZE\n");

    return STATUS_SUCCESS;

Abort_teardown_firewall:

    firewall::TearDown(&context->Firewall);

Abort_teardown_process_registry:

    DestroyProcessRegistryMgmt(&context->ProcessRegistry);

Abort_teardown_registeredimage:

    registeredimage::TearDown((registeredimage::CONTEXT**)&context->RegisteredImage.Instance);

Abort_teardown_procbroker:

    procbroker::TearDown(&context->ProcessEventBroker);

Abort_teardown_eventing:

    eventing::TearDown(&context->Eventing);

    return status;
}

//
// SetConfigurationPrepare()
//
// Validate and repackage configuration data into new registered image instance.
//
// This runs at PASSIVE, in order to be able to downcase the strings.
//
NTSTATUS
SetConfigurationPrepare(WDFREQUEST Request, registeredimage::CONTEXT** Imageset) {
    *Imageset = NULL;

    PVOID buffer;
    size_t bufferLength;

    auto status =
        WdfRequestRetrieveInputBuffer(Request, (size_t)MIN_REQUEST_SIZE::SET_CONFIGURATION, &buffer, &bufferLength);

    if (!NT_SUCCESS(status)) {
        DbgPrint("Could not access configuration buffer provided to IOCTL: 0x%X\n", status);

        return status;
    }

    if (!ValidateUserBufferConfiguration(buffer, bufferLength)) {
        DbgPrint("Invalid configuration data in buffer provided to IOCTL\n");

        return STATUS_INVALID_PARAMETER;
    }

    auto header = (ST_CONFIGURATION_HEADER*)buffer;
    auto entry = (ST_CONFIGURATION_ENTRY*)(header + 1);
    auto stringBuffer = (UCHAR*)(entry + header->NumEntries);

    if (header->NumEntries == 0) {
        DbgPrint("Cannot assign empty configuration\n");

        return STATUS_INVALID_PARAMETER;
    }

    //
    // Create a new instance for storing all image names (excluded and hybrid).
    //

    registeredimage::CONTEXT* imageset;

    status = registeredimage::Initialize(&imageset, ST_PAGEABLE::NO);

    if (!NT_SUCCESS(status)) {
        DbgPrint("Could not create new registered image instance: 0x%X\n", status);

        return status;
    }

    //
    // Insert each entry with its flags so callers can distinguish excluded from hybrid.
    //

    for (auto i = 0; i < header->NumEntries; ++i, ++entry) {
        UNICODE_STRING s;

        s.Length = entry->ImageNameLength;
        s.MaximumLength = entry->ImageNameLength;
        s.Buffer = (WCHAR*)(stringBuffer + entry->ImageNameOffset);

        status = registeredimage::AddEntry(imageset, &s, entry->Flags);

        if (!NT_SUCCESS(status)) {
            DbgPrint("Could not insert new entry into registered image instance: 0x%X\n", status);

            registeredimage::TearDown(&imageset);

            return status;
        }
    }

    *Imageset = imageset;

    return STATUS_SUCCESS;
}

//
// SetConfiguration()
//
// Store updated configuration.
//
// Possibly enter/leave engaged state depending on a number of factors.
//
NTSTATUS
SetConfiguration(WDFDEVICE Device, registeredimage::CONTEXT* Imageset) {
    auto context = DeviceGetSplitTunnelContext(Device);

    NTSTATUS status = STATUS_UNSUCCESSFUL;

    WdfWaitLockAcquire(context->DriverState.Lock, NULL);

    switch (context->DriverState.State) {
    case ST_DRIVER_STATE_READY: {
        status = RegisterConfigurationAtReady(context, Imageset);

        break;
    }
    case ST_DRIVER_STATE_ENGAGED: {
        status = RegisterConfigurationAtEngaged(context, Imageset);

        break;
    }
    }

    WdfWaitLockRelease(context->DriverState.Lock);

    if (NT_SUCCESS(status)) {
        DbgPrint("Successfully processed IOCTL_ST_SET_CONFIGURATION\n");

        //
        // No locking required since we're in a serialized IOCTL handler path.
        //
        registeredimage::ForEach(context->RegisteredImage.Instance, DbgPrintConfiguration, NULL);
    }

    return status;
}

//
// GetConfigurationComplete()
//
// Return current configuration to driver client.
//
// Locking is not required for the following reasons:
//
// - We're in the serialized IOCTL handler path.
// - Config is only read from, not written to.
//
void GetConfigurationComplete(WDFDEVICE Device, WDFREQUEST Request) {
    PVOID buffer;
    size_t bufferLength;

    auto status =
        WdfRequestRetrieveOutputBuffer(Request, (size_t)MIN_REQUEST_SIZE::GET_CONFIGURATION, &buffer, &bufferLength);

    if (!NT_SUCCESS(status)) {
        WdfRequestComplete(Request, status);

        return;
    }

    //
    // Buffer is present and meets the minimum size requirements.
    // This means we can "complete with information".
    //

    ULONG_PTR info = 0;

    //
    // Compute required buffer length.
    //

    auto context = DeviceGetSplitTunnelContext(Device);

    CONFIGURATION_COMPUTE_LENGTH_CONTEXT computeContext;

    computeContext.NumEntries = 0;
    computeContext.TotalStringLength = 0;

    registeredimage::ForEach(context->RegisteredImage.Instance, GetConfigurationComputeLength, &computeContext);

    SIZE_T requiredLength = sizeof(ST_CONFIGURATION_HEADER) +
                            (sizeof(ST_CONFIGURATION_ENTRY) * computeContext.NumEntries) +
                            computeContext.TotalStringLength;

    //
    // It's not possible to fail the request AND provide output data.
    //
    // Therefore, the only two types of valid input buffers are:
    //
    // # A buffer large enough to contain the settings.
    // # A buffer of exactly sizeof(SIZE_T) bytes, to learn the required length.
    //

    if (bufferLength < requiredLength) {
        if (bufferLength == sizeof(SIZE_T)) {
            status = STATUS_SUCCESS;

            *(SIZE_T*)buffer = requiredLength;

            info = sizeof(SIZE_T);
        } else {
            status = STATUS_BUFFER_TOO_SMALL;

            info = 0;
        }

        WdfRequestCompleteWithInformation(Request, status, info);

        return;
    }

    //
    // Output buffer is OK.
    // Serialize config into buffer.
    //

    auto header = (ST_CONFIGURATION_HEADER*)buffer;
    auto entry = (ST_CONFIGURATION_ENTRY*)(header + 1);
    auto stringBuffer = (UCHAR*)(entry + computeContext.NumEntries);

    CONFIGURATION_SERIALIZE_CONTEXT serializeContext;

    serializeContext.Entry = entry;
    serializeContext.StringOffset = 0;
    serializeContext.StringDest = stringBuffer;

    registeredimage::ForEach(context->RegisteredImage.Instance, GetConfigurationSerialize, &serializeContext);

    //
    // Finalize header.
    //

    header->NumEntries = computeContext.NumEntries;
    header->TotalLength = requiredLength;

    info = requiredLength;

    status = STATUS_SUCCESS;

    WdfRequestCompleteWithInformation(Request, status, info);
}

//
// ClearConfiguration()
//
// Mark all processes as non-split and clear configuration.
//
NTSTATUS
ClearConfiguration(WDFDEVICE Device) {
    auto context = DeviceGetSplitTunnelContext(Device);

    WdfWaitLockAcquire(context->DriverState.Lock, NULL);

    if (context->DriverState.State == ST_DRIVER_STATE_ENGAGED) {
        //
        // Leave engaged state.
        // (This updates the process registry and sends splitting events.)
        //

        auto status = LeaveEngagedState(context);

        if (!NT_SUCCESS(status)) {
            WdfWaitLockRelease(context->DriverState.Lock);

            DbgPrint("Could not leave engaged state: 0x%X\n", status);

            return status;
        }
    }

    registeredimage::Reset(context->RegisteredImage.Instance);

    WdfWaitLockRelease(context->DriverState.Lock);

    DbgPrint("Successfully processed IOCTL_ST_CLEAR_CONFIGURATION\n");

    return STATUS_SUCCESS;
}

NTSTATUS
RegisterProcesses(WDFDEVICE Device, WDFREQUEST Request) {
    PVOID buffer;
    size_t bufferLength;

    auto status =
        WdfRequestRetrieveInputBuffer(Request, (size_t)MIN_REQUEST_SIZE::REGISTER_PROCESSES, &buffer, &bufferLength);

    if (!NT_SUCCESS(status)) {
        return status;
    }

    if (!ValidateUserBufferProcesses(buffer, bufferLength)) {
        DbgPrint("Invalid data provided to IOCTL_ST_REGISTER_PROCESSES\n");

        return STATUS_INVALID_PARAMETER;
    }

    auto header = (ST_PROCESS_DISCOVERY_HEADER*)buffer;
    auto entry = (ST_PROCESS_DISCOVERY_ENTRY*)(header + 1);
    auto stringBuffer = (UCHAR*)(entry + header->NumEntries);

    auto context = DeviceGetSplitTunnelContext(Device);

    NT_ASSERT(procregistry::IsEmpty(context->ProcessRegistry.Instance));

    //
    // Insert records one by one.
    //
    // We can't check the configuration to get accurate information on whether the process being
    // inserted should have its traffic split.
    //
    // Because there is no configuration yet.
    //

    for (auto i = 0; i < header->NumEntries; ++i, ++entry) {
        UNICODE_STRING imagename;

        imagename.Length = entry->ImageNameLength;
        imagename.MaximumLength = entry->ImageNameLength;

        if (entry->ImageNameLength == 0) {
            imagename.Buffer = NULL;
        } else {
            imagename.Buffer = (WCHAR*)(stringBuffer + entry->ImageNameOffset);
        }

        procregistry::PROCESS_REGISTRY_ENTRY registryEntry = { 0 };

        status = procregistry::InitializeEntry(
            context->ProcessRegistry.Instance,
            entry->ParentProcessId,
            entry->ProcessId,
            ST_PROCESS_SPLIT_STATUS_OFF,
            &imagename,
            &registryEntry);

        if (!NT_SUCCESS(status)) {
            procregistry::Reset(context->ProcessRegistry.Instance);

            return status;
        }

        status = procregistry::AddEntry(context->ProcessRegistry.Instance, &registryEntry);

        if (!NT_SUCCESS(status)) {
            procregistry::ReleaseEntry(&registryEntry);

            procregistry::Reset(context->ProcessRegistry.Instance);

            return status;
        }
    }

    context->DriverState.State = ST_DRIVER_STATE_READY;

    procmgmt::Activate(context->ProcessMgmt);

    DbgPrint("Successfully processed IOCTL_ST_REGISTER_PROCESSES\n");

    return STATUS_SUCCESS;
}

//
// RegisterIpAddresses()
//
// Store updated set of IP addresses.
//
// Possibly enter/leave engaged state depending on a number of factors.
//
NTSTATUS
RegisterIpAddresses(WDFDEVICE Device, WDFREQUEST Request) {
    PVOID buffer;
    size_t bufferLength;

    auto status =
        WdfRequestRetrieveInputBuffer(Request, (size_t)MIN_REQUEST_SIZE::REGISTER_IP_ADDRESSES, &buffer, &bufferLength);

    if (!NT_SUCCESS(status)) {
        return status;
    }

    if (bufferLength != sizeof(ST_IP_ADDRESSES)) {
        DbgPrint("Invalid data provided to IOCTL_ST_REGISTER_IP_ADDRESSES\n");

        return STATUS_INVALID_PARAMETER;
    }

    auto newIpAddresses = (ST_IP_ADDRESSES*)buffer;

    //
    // New addresses seem OK, branch on current state.
    //

    status = STATUS_UNSUCCESSFUL;

    auto context = DeviceGetSplitTunnelContext(Device);

    WdfWaitLockAcquire(context->DriverState.Lock, NULL);

    switch (context->DriverState.State) {
    case ST_DRIVER_STATE_READY: {
        status = RegisterIpAddressesAtReady(context, newIpAddresses);

        break;
    }
    case ST_DRIVER_STATE_ENGAGED: {
        status = RegisterIpAddressesAtEngaged(context, newIpAddresses);

        break;
    }
    }

    WdfWaitLockRelease(context->DriverState.Lock);

    if (NT_SUCCESS(status)) {
        DbgPrint("Successfully processed IOCTL_ST_REGISTER_IP_ADDRESSES\n");
    }

    return status;
}

//
// GetIpAddressesComplete()
//
// Return currently registered IP addresses to driver client.
//
// Locking is not required for the following reasons:
//
// - We're in the serialized IOCTL handler path.
// - IP addresses struct is only read from, not written to.
//
void GetIpAddressesComplete(WDFDEVICE Device, WDFREQUEST Request) {
    NT_ASSERT((size_t)MIN_REQUEST_SIZE::GET_IP_ADDRESSES >= sizeof(ST_IP_ADDRESSES));

    PVOID buffer;

    auto status = WdfRequestRetrieveOutputBuffer(Request, (size_t)MIN_REQUEST_SIZE::GET_IP_ADDRESSES, &buffer, NULL);

    if (!NT_SUCCESS(status)) {
        WdfRequestComplete(Request, status);

        return;
    }

    //
    // Copy IP addresses struct to output buffer.
    //

    auto context = DeviceGetSplitTunnelContext(Device);

    RtlCopyMemory(buffer, &context->IpAddresses, sizeof(context->IpAddresses));

    //
    // Finish up.
    //

    WdfRequestCompleteWithInformation(Request, STATUS_SUCCESS, sizeof(context->IpAddresses));
}

void GetStateComplete(WDFDEVICE Device, WDFREQUEST Request) {
    PVOID buffer;

    auto status = WdfRequestRetrieveOutputBuffer(Request, (size_t)MIN_REQUEST_SIZE::GET_STATE, &buffer, NULL);

    if (!NT_SUCCESS(status)) {
        DbgPrint("Unable to retrieve client buffer or invalid buffer size\n");

        WdfRequestComplete(Request, status);

        return;
    }

    auto context = DeviceGetSplitTunnelContext(Device);

    // Sample current state.
    *(SIZE_T*)buffer = context->DriverState.State;

    WdfRequestCompleteWithInformation(Request, STATUS_SUCCESS, sizeof(SIZE_T));
}

//
// QueryProcessComplete()
//
// Returns information about specific process to driver client.
//
void QueryProcessComplete(WDFDEVICE Device, WDFREQUEST Request) {
    PVOID buffer;
    size_t bufferLength;

    auto status =
        WdfRequestRetrieveInputBuffer(Request, (size_t)MIN_REQUEST_SIZE::QUERY_PROCESS, &buffer, &bufferLength);

    if (!NT_SUCCESS(status)) {
        DbgPrint("Unable to retrieve input buffer or buffer too small\n");

        WdfRequestComplete(Request, status);

        return;
    }

    if (bufferLength != (size_t)MIN_REQUEST_SIZE::QUERY_PROCESS) {
        DbgPrint("Invalid buffer size\n");

        WdfRequestComplete(Request, STATUS_INVALID_BUFFER_SIZE);

        return;
    }

    auto processId = ((ST_QUERY_PROCESS*)buffer)->ProcessId;

    //
    // Get the output buffer.
    //
    // We can't validate the buffer length just yet, because we don't know the
    // length of the process image name.
    //

    status = WdfRequestRetrieveOutputBuffer(
        Request, (size_t)MIN_REQUEST_SIZE::QUERY_PROCESS_RESPONSE, &buffer, &bufferLength);

    if (!NT_SUCCESS(status)) {
        DbgPrint("Unable to retrieve output buffer or buffer too small\n");

        WdfRequestComplete(Request, status);

        return;
    }

    //
    // Look up process.
    //

    auto context = DeviceGetSplitTunnelContext(Device);

    WdfSpinLockAcquire(context->ProcessRegistry.Lock);

    auto record = procregistry::FindEntry(context->ProcessRegistry.Instance, processId);

    if (record == NULL) {
        WdfSpinLockRelease(context->ProcessRegistry.Lock);

        DbgPrint("Process query for unknown process\n");

        WdfRequestComplete(Request, STATUS_INVALID_HANDLE);

        return;
    }

    //
    // Definitively validate output buffer.
    //

    auto requiredLength = sizeof(ST_QUERY_PROCESS_RESPONSE) - RTL_FIELD_SIZE(ST_QUERY_PROCESS_RESPONSE, ImageName) +
                          record->ImageName.Length;

    if (bufferLength < requiredLength) {
        WdfSpinLockRelease(context->ProcessRegistry.Lock);

        DbgPrint("Output buffer is too small\n");

        WdfRequestComplete(Request, STATUS_BUFFER_TOO_SMALL);

        return;
    }

    //
    // Copy data and release lock.
    //

    auto response = (ST_QUERY_PROCESS_RESPONSE*)buffer;

    response->ProcessId = record->ProcessId;
    response->ParentProcessId = record->ParentProcessId;
    response->Split = (util::SplittingEnabled(record->Settings.Split) ? TRUE : FALSE);
    response->ImageNameLength = record->ImageName.Length;

    RtlCopyMemory(&response->ImageName, record->ImageName.Buffer, record->ImageName.Length);

    WdfSpinLockRelease(context->ProcessRegistry.Lock);

    //
    // Complete request.
    //

    WdfRequestCompleteWithInformation(Request, STATUS_SUCCESS, requiredLength);
}

void ResetComplete(WDFDEVICE Device, WDFREQUEST Request) {
    //
    // We're in the serialized IOCTL handler path so handlers that might update the state are
    // locked out from executing.
    //
    // That's the first reason to not acquire the state lock.
    //
    // The second reason is that process management logic uses the state lock so we can't be
    // holding the lock while trying to tear down the process management subsystem.
    //

    WdfRequestComplete(Request, Reset(Device));
}

NTSTATUS
Reset(WDFDEVICE Device) {
    auto context = DeviceGetSplitTunnelContext(Device);

    NTSTATUS status = STATUS_SUCCESS;

    switch (context->DriverState.State) {
    case ST_DRIVER_STATE_STARTED: {
        break;
    }
    case ST_DRIVER_STATE_ZOMBIE: {
        DbgPrint("Rejecting reset in zombie state\n");
        status = STATUS_CANCELLED;
        break;
    }
    default: {
        status = ResetInner(context);
    }
    }

    if (NT_SUCCESS(status)) {
        DbgPrint("Successfully processed IOCTL_ST_RESET\n");
    } else {
        DbgPrint("Failed to reset driver state\n");
    }

    return status;
}

} // namespace ioctl
