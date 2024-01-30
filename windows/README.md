## Windows dependencies

In this directory are located sources of required libraries for
Windows build

### Prerequisites

- Microsoft's Visual Studio Build Tools 2022 (a regular installation of Visual Studio 2022 Community or Pro edition works as well)
- Windows 10/11 SDK (check Visual Studio Installer)
- `MSBuild.exe` available in `PATH`. If installed it via Visual Studio, the binary can be found under:

```
C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\MSBuild\Current\Bin
```

- `bash`, provided by [Git for Windows](https://git-scm.com/download/win)

### Build

Under a Windows environment (likely PowerShell), switch to `bash`
then run the following script

```shell
bash
./build-windows-modules.sh
```

