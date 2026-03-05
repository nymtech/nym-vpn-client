// Copyright 2016-2026 Mullvad VPN AB. All Rights Reserved.
// Copyright 2026 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

#include "win64guard.h"

#include <ntddk.h>
#include <wdf.h>
#include <wdmsec.h>
#include <mstcpip.h>

#include "defs/ioctl.h"
#include "devicecontext.h"
#include "eventing/eventing.h"
#include "firewall/firewall.h"
#include "ioctl.h"
#include "util.h"

#include "trace.h"
#include "driverentry.tmh"

extern "C" DRIVER_INITIALIZE DriverEntry;

extern "C" // Because alloc_text requires this.
    NTSTATUS StCreateDevice(IN WDFDRIVER WdfDriver);

EVT_WDF_IO_QUEUE_IO_DEVICE_CONTROL StEvtIoDeviceControl;
EVT_WDF_IO_QUEUE_IO_DEVICE_CONTROL StEvtIoDeviceControlParallel;
EVT_WDF_IO_QUEUE_IO_DEVICE_CONTROL StEvtIoDeviceControlSerial;

EVT_WDF_DRIVER_UNLOAD StEvtDriverUnload;

#pragma alloc_text(INIT, DriverEntry)
#pragma alloc_text(INIT, StCreateDevice)

#define ST_DEVICE_SECURITY_DESCRIPTOR SDDL_DEVOBJ_SYS_ALL_ADM_ALL

#define ST_DEVICE_NAME_STRING L"\\Device\\NYMVPNSPLITTUNNEL"
#define ST_SYMBOLIC_NAME_STRING L"\\Global??\\NYMVPNSPLITTUNNEL"

constexpr size_t MAX_IO_BUFFER_SIZE = 100000000; // 100 MB

namespace {

//
// RaiseDispatchForwardRequest()
//
// Raise to DISPATCH level and forward to IO queue.
//
// As it turns out, WdfRequestForwardToIoQueue() will borrow the calling thread
// to service the queue whenever it determines this is more efficient.
//
// This becomes a problem if we're in our topmost IOCTL handler and are trying
// to forward the request so we can return and unblock the client.
//
// If the destination queue is configured to service requests at PASSIVE, we can
// raise to DISPATCH to prevent our thread from being borrowed :-)
//
NTSTATUS
RaiseDispatchForwardRequest(WDFREQUEST Request, WDFQUEUE Queue) {
    auto const oldIrql = KeRaiseIrqlToDpcLevel();

    auto const status = WdfRequestForwardToIoQueue(Request, Queue);

    KeLowerIrql(oldIrql);

    return status;
}

//
// IoControlRequiresParallelProcessing()
//
// Evaluate whether `IoControlCode` uses inverted call.
//
bool IoControlRequiresParallelProcessing(ULONG IoControlCode) {
    return IoControlCode == IOCTL_ST_DEQUEUE_EVENT;
}

//
// If the driver is unloaded without being properly reset first, we must do our
// best to try to clean up non-device related resources.
//
// Sadly, other cleanup routines for specifically the device appear not to run
// in these cases, so we access it through a global variable.
//
WDFDEVICE g_wdfDevice = nullptr;

} // anonymous namespace

//
// DriverEntry
//
// Creates a single device with associated symbolic link.
// Does minimal initialization.
//
extern "C" NTSTATUS DriverEntry(_In_ PDRIVER_OBJECT DriverObject, _In_ PUNICODE_STRING RegistryPath) {
    WPP_INIT_TRACING(DriverObject, RegistryPath);

    DbgPrint("Loading Nym VPN Split Tunnel driver\n");

    ExInitializeDriverRuntime(DrvRtPoolNxOptIn);

    //
    // Create WDF driver object.
    //

    WDF_DRIVER_CONFIG config;

    WDF_DRIVER_CONFIG_INIT(&config, WDF_NO_EVENT_CALLBACK);

    config.DriverInitFlags |= WdfDriverInitNonPnpDriver;
    config.EvtDriverUnload = StEvtDriverUnload;
    config.DriverPoolTag = ST_POOL_TAG;

    WDFDRIVER wdfDriver;

    auto status = WdfDriverCreate(DriverObject, RegistryPath, WDF_NO_OBJECT_ATTRIBUTES, &config, &wdfDriver);

    if (!NT_SUCCESS(status)) {
        DbgPrint("WdfDriverCreate() failed 0x%X\n", status);

        //
        // StEvtDriverUnload() won't be called so we have to
        // clean up WPP here instead.
        //

        WPP_CLEANUP(DriverObject);

        return status;
    }

    //
    // Create WDF device object.
    //

    status = StCreateDevice(wdfDriver);

    if (!NT_SUCCESS(status)) {
        DbgPrint("StCreateDevice() failed 0x%X\n", status);
        return status;
    }

    //
    // All set.
    //

    DbgPrint("Successfully loaded Nym VPN Split Tunnel driver\n");

    return STATUS_SUCCESS;
}

extern "C" NTSTATUS StCreateDevice(IN WDFDRIVER WdfDriver) {
    DECLARE_CONST_UNICODE_STRING(deviceName, ST_DEVICE_NAME_STRING);
    DECLARE_CONST_UNICODE_STRING(symbolicLinkName, ST_SYMBOLIC_NAME_STRING);

    WDF_OBJECT_ATTRIBUTES attributes;
    WDFDEVICE wdfDevice = nullptr;
    WDF_IO_QUEUE_CONFIG queueConfig;
    WDFQUEUE parallelQueue = nullptr;
    WDFQUEUE serialQueue = nullptr;
    ST_DEVICE_CONTEXT* context = nullptr;

    auto deviceInit = WdfControlDeviceInitAllocate(WdfDriver, &ST_DEVICE_SECURITY_DESCRIPTOR);

    if (deviceInit == NULL) {
        DbgPrint("WdfControlDeviceInitAllocate() failed\n");
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    WdfDeviceInitSetExclusive(deviceInit, TRUE);

    auto status = WdfDeviceInitAssignName(deviceInit, &deviceName);

    if (!NT_SUCCESS(status)) {
        DbgPrint("WdfDeviceInitAssignName() failed 0x%X\n", status);
        goto Cleanup;
    }

    //
    // No need to call WdfDeviceInitSetIoType() that configures the I/O type for
    // read and write requests.
    //
    // We're using IOCTL for everything, which have the I/O type encoded.
    //
    // ---
    //
    // No need to call WdfControlDeviceInitSetShutdownNotification() because
    // we don't care about the system being shut down.
    //
    // ---
    //
    // No need to call WdfDeviceInitSetFileObjectConfig() because we're not
    // interested in receiving events when device handles are created/destroyed.
    //
    // --
    //
    // No need to call WdfDeviceInitSetIoInCallerContextCallback() because
    // we're not using METHOD_NEITHER for any buffers.
    //

    WDF_OBJECT_ATTRIBUTES_INIT_CONTEXT_TYPE(&attributes, ST_DEVICE_CONTEXT);

    status = WdfDeviceCreate(&deviceInit, &attributes, &wdfDevice);

    if (!NT_SUCCESS(status)) {
        DbgPrint("WdfDeviceCreate() failed 0x%X\n", status);
        goto Cleanup;
    }

    status = WdfDeviceCreateSymbolicLink(wdfDevice, &symbolicLinkName);

    if (!NT_SUCCESS(status)) {
        DbgPrint("WdfDeviceCreateSymbolicLink() failed 0x%X\n", status);
        goto Cleanup;
    }

    //
    // Create a default request queue.
    // Only register to handle IOCTL requests.
    // Use WdfIoQueueDispatchParallel to enable inverted call.
    //

    WDF_IO_QUEUE_CONFIG_INIT_DEFAULT_QUEUE(&queueConfig, WdfIoQueueDispatchParallel);

    queueConfig.EvtIoDeviceControl = StEvtIoDeviceControl;
    queueConfig.PowerManaged = WdfFalse;

    WDF_OBJECT_ATTRIBUTES_INIT(&attributes);
    attributes.ExecutionLevel = WdfExecutionLevelPassive;

    status = WdfIoQueueCreate(wdfDevice, &queueConfig, &attributes, WDF_NO_HANDLE);

    if (!NT_SUCCESS(status)) {
        DbgPrint("WdfIoQueueCreate() for default queue failed 0x%X\n", status);
        goto Cleanup;
    }

    //
    // Create a secondary queue which is also using parallel dispatching.
    // This enables us to forward incoming requests and return before processing
    // completes.
    //

    WDF_IO_QUEUE_CONFIG_INIT(&queueConfig, WdfIoQueueDispatchParallel);

    queueConfig.EvtIoDeviceControl = StEvtIoDeviceControlParallel;
    queueConfig.PowerManaged = WdfFalse;

    WDF_OBJECT_ATTRIBUTES_INIT(&attributes);
    attributes.ExecutionLevel = WdfExecutionLevelPassive;

    status = WdfIoQueueCreate(wdfDevice, &queueConfig, &attributes, &parallelQueue);

    if (!NT_SUCCESS(status)) {
        DbgPrint("WdfIoQueueCreate() for parallel queue failed 0x%X\n", status);
        goto Cleanup;
    }

    //
    // Create a third queue that uses serialized dispatching.
    // Commands that need to be serialized can then be forwarded to this queue.
    //

    WDF_IO_QUEUE_CONFIG_INIT(&queueConfig, WdfIoQueueDispatchSequential);

    queueConfig.EvtIoDeviceControl = StEvtIoDeviceControlSerial;
    queueConfig.PowerManaged = WdfFalse;

    WDF_OBJECT_ATTRIBUTES_INIT(&attributes);
    attributes.ExecutionLevel = WdfExecutionLevelPassive;

    status = WdfIoQueueCreate(wdfDevice, &queueConfig, &attributes, &serialQueue);

    if (!NT_SUCCESS(status)) {
        DbgPrint("WdfIoQueueCreate() for serialized queue failed 0x%X\n", status);
        goto Cleanup;
    }

    //
    // Initialize context.
    //

    context = DeviceGetSplitTunnelContext(wdfDevice);

    RtlZeroMemory(context, sizeof(*context));

    status = WdfWaitLockCreate(WDF_NO_OBJECT_ATTRIBUTES, &context->DriverState.Lock);

    if (!NT_SUCCESS(status)) {
        DbgPrint("WdfWaitLockCreate() failed 0x%X\n", status);
        goto Cleanup;
    }

    context->DriverState.State = ST_DRIVER_STATE_STARTED;

    context->ParallelRequestQueue = parallelQueue;
    context->SerializedRequestQueue = serialQueue;

    g_wdfDevice = wdfDevice;

    WdfControlFinishInitializing(wdfDevice);

    status = STATUS_SUCCESS;

Cleanup:

    if (deviceInit != NULL) {
        WdfDeviceInitFree(deviceInit);
    }

    return status;
}

VOID StEvtIoDeviceControl(
    WDFQUEUE Queue, WDFREQUEST Request, size_t OutputBufferLength, size_t InputBufferLength, ULONG IoControlCode) {
    //
    // Check that the input/output buffers aren't unreasonably large to
    // disallow userspace from exhausting kernel memory.
    //

    if (InputBufferLength > MAX_IO_BUFFER_SIZE) {
        DbgPrint("Input buffer is too big. IOCTL=%lu InputBufferLength=%llu\n", IoControlCode, InputBufferLength);
        WdfRequestComplete(Request, STATUS_INVALID_PARAMETER);
        return;
    }
    if (OutputBufferLength > MAX_IO_BUFFER_SIZE) {
        DbgPrint("Output buffer is too big. IOCTL=%lu OutputBufferLength=%llu\n", IoControlCode, OutputBufferLength);
        WdfRequestComplete(Request, STATUS_INVALID_PARAMETER);
        return;
    }

    auto device = WdfIoQueueGetDevice(Queue);
    auto context = DeviceGetSplitTunnelContext(device);

    //
    // Querying the current driver state is always a valid operation, regardless
    // of state. This is safe to service inline because it doesn't acquire any
    // locks.
    //

    if (IoControlCode == IOCTL_ST_GET_STATE) {
        ioctl::GetStateComplete(device, Request);

        return;
    }

    //
    // Select which queue the request is forwarded to.
    //

    auto targetQueue = IoControlRequiresParallelProcessing(IoControlCode) ? context->ParallelRequestQueue
                                                                          : context->SerializedRequestQueue;

    auto const status = RaiseDispatchForwardRequest(Request, targetQueue);

    if (NT_SUCCESS(status)) {
        return;
    }

    DbgPrint("Failed to forward request to secondary IOCTL queue\n");

    WdfRequestComplete(Request, status);
}

bool StEvtIoDeviceControlParallelInner(WDFREQUEST Request, ULONG IoControlCode, ST_DEVICE_CONTEXT* Context) {
    switch (IoControlCode) {
    case IOCTL_ST_DEQUEUE_EVENT: {
        //
        // TODO: This approach is slightly broken.
        //
        // CollectOne() may enqueue the request in anticipation of an event
        // arriving. That means the request completion may come at a later time when
        // the asserted driver state has changed.
        //
        // But this probably doesn't matter.
        //

        if (Context->DriverState.State >= ST_DRIVER_STATE_INITIALIZED &&
            Context->DriverState.State <= ST_DRIVER_STATE_ENGAGED) {
            eventing::CollectOne(Context->Eventing, Request);

            return true;
        }

        break;
    }
    };

    return false;
}

VOID StEvtIoDeviceControlParallel(
    WDFQUEUE Queue, WDFREQUEST Request, size_t OutputBufferLength, size_t InputBufferLength, ULONG IoControlCode) {
    UNREFERENCED_PARAMETER(OutputBufferLength);
    UNREFERENCED_PARAMETER(InputBufferLength);

    auto device = WdfIoQueueGetDevice(Queue);
    auto context = DeviceGetSplitTunnelContext(device);

    //
    // Keep state lock acquired for the duration of processing.
    // This prevents serialized IOCTL handlers from transitioning the state.
    //

    WdfWaitLockAcquire(context->DriverState.Lock, NULL);

    bool servicedRequest = StEvtIoDeviceControlParallelInner(Request, IoControlCode, context);

    WdfWaitLockRelease(context->DriverState.Lock);

    if (servicedRequest) {
        return;
    }

    DbgPrint("Invalid IOCTL or not valid for current driver state\n");

    WdfRequestComplete(Request, STATUS_INVALID_DEVICE_REQUEST);
}

VOID StEvtIoDeviceControlSerial(
    WDFQUEUE Queue, WDFREQUEST Request, size_t OutputBufferLength, size_t InputBufferLength, ULONG IoControlCode) {
    UNREFERENCED_PARAMETER(Queue);
    UNREFERENCED_PARAMETER(OutputBufferLength);
    UNREFERENCED_PARAMETER(InputBufferLength);

    auto device = WdfIoQueueGetDevice(Queue);

    if (IoControlCode == IOCTL_ST_RESET) {
        //
        // Potential state transition here.
        //
        ioctl::ResetComplete(device, Request);

        return;
    }

    auto context = DeviceGetSplitTunnelContext(device);

    switch (context->DriverState.State) {
    case ST_DRIVER_STATE_STARTED: {
        //
        // Valid controls:
        //
        // IOCTL_ST_INITIALIZE
        //

        if (IoControlCode == IOCTL_ST_INITIALIZE) {
            //
            // Definitive state transition here.
            // No locking needed this early.
            //
            WdfRequestComplete(Request, ioctl::Initialize(device));

            return;
        }

        break;
    }
    case ST_DRIVER_STATE_INITIALIZED: {
        //
        // Valid controls:
        //
        // IOCTL_ST_REGISTER_PROCESSES
        //

        if (IoControlCode == IOCTL_ST_REGISTER_PROCESSES) {
            //
            // Definitive state transition here.
            // No locking needed this early.
            //
            WdfRequestComplete(Request, ioctl::RegisterProcesses(device, Request));

            return;
        }

        break;
    }
    case ST_DRIVER_STATE_READY:
    case ST_DRIVER_STATE_ENGAGED: {
        //
        // Valid controls:
        //
        // IOCTL_ST_REGISTER_IP_ADDRESSES
        // IOCTL_ST_GET_IP_ADDRESSES
        // IOCTL_ST_SET_CONFIGURATION
        // IOCTL_ST_GET_CONFIGURATION
        // IOCTL_ST_CLEAR_CONFIGURATION
        // IOCTL_ST_QUERY_PROCESS
        //

        if (IoControlCode == IOCTL_ST_REGISTER_IP_ADDRESSES) {
            //
            // Potential state transition here.
            //
            auto status = ioctl::RegisterIpAddresses(device, Request);

            WdfRequestComplete(Request, status);

            return;
        }

        if (IoControlCode == IOCTL_ST_GET_IP_ADDRESSES) {
            ioctl::GetIpAddressesComplete(device, Request);

            return;
        }

        if (IoControlCode == IOCTL_ST_SET_CONFIGURATION) {
            registeredimage::CONTEXT* imageset;

            auto status = ioctl::SetConfigurationPrepare(Request, &imageset);

            if (!NT_SUCCESS(status)) {
                WdfRequestComplete(Request, status);

                return;
            }

            //
            // Potential state transition here.
            //
            status = ioctl::SetConfiguration(device, imageset);

            WdfRequestComplete(Request, status);

            return;
        }

        if (IoControlCode == IOCTL_ST_GET_CONFIGURATION) {
            ioctl::GetConfigurationComplete(device, Request);

            return;
        }

        if (IoControlCode == IOCTL_ST_CLEAR_CONFIGURATION) {
            //
            // Potential state transition here.
            //
            auto status = ioctl::ClearConfiguration(device);

            WdfRequestComplete(Request, status);

            return;
        }

        if (IoControlCode == IOCTL_ST_QUERY_PROCESS) {
            ioctl::QueryProcessComplete(device, Request);

            return;
        }

        break;
    }
    case ST_DRIVER_STATE_ZOMBIE: {
        DbgPrint("Zombie state: Rejecting all requests\n");

        WdfRequestComplete(Request, STATUS_CANCELLED);

        return;
    }
    }

    DbgPrint("Invalid IOCTL or not valid for current driver state\n");

    WdfRequestComplete(Request, STATUS_INVALID_DEVICE_REQUEST);
}

VOID StEvtDriverUnload(IN WDFDRIVER WdfDriver) {
    UNREFERENCED_PARAMETER(WdfDriver);

    DbgPrint("Unloading Nym VPN Split Tunnel driver\n");

    // The device object is necessarily set because this runs only if DriverEntry
    // succeeded.
    NT_ASSERT(g_wdfDevice != nullptr);
    auto context = DeviceGetSplitTunnelContext(g_wdfDevice);

    if (ST_DRIVER_STATE_STARTED != context->DriverState.State) {
        // We should never end up here if the driver is properly reset first,
        // since we cannot guarantee that callouts will be successfully
        // unregistered. But just in case we do, we attempt to clean up anyway.

        // We can assume that no IOCTL requests are pending here because the driver
        // will not unload if there is a handle open to our control device object.

        DbgPrint("Resetting driver during unload\n");
        auto const status = ioctl::Reset(g_wdfDevice);

        // If resetting fails, we are out of luck.
        if (status != STATUS_SUCCESS) {
            KeBugCheckEx(MANUALLY_INITIATED_CRASH1, 0, 0, 0, 0);
        }
    }

    WPP_CLEANUP(WdfDriverWdmGetDriverObject(WdfDriver));
}
