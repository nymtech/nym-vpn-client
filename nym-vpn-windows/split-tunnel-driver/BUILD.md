# Compiling the split tunnel driver

## Prerequisites

### VS 2022 and WDK

As per [this Microsoft article](https://learn.microsoft.com/en-us/windows-hardware/drivers/gettingstarted/writing-a-kmdf-driver-based-on-a-templat), install Visual Studio 2022 with the "Desktop development with C++" workload, and the following individual components:

- MSVC v143 - VS 2022 C++ ARM64/ARM64EC Spectre-mitigated libs (Latest)
- MSVC v143 - VS 2022 C++ x64/x86 Spectre-mitigated libs (Latest)
- C++ ATL for latest v143 build tools with Spectre Mitigations (ARM64/ARM64EC)
- C++ ATL for latest v143 build tools with Spectre Mitigations (x86 & x64)
- C++ MFC for latest v143 build tools with Spectre Mitigations (ARM64/ARM64EC)
- C++ MFC for latest v143 build tools with Spectre Mitigations (x86 & x64)
- Windows Driver Kit

Note that the Windows Driver Kit will install _most_ of the WDK files, however not all of them. It will, however, install a VS 2022 Extension that is required to configure the driver. To install the complete WDK, use:

```
winget install --id Microsoft.WindowsWDK.10.0.26100
```

### Clang Build Tools Extension

This extension is useful for handle bulk reformatting of the source files. You can find it via `Manage Extensions` in Visual Studio, or install it via:
