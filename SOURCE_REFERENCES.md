# SOURCE_REFERENCES

Bounded source references used to shape emulator behavior. These are evidence
anchors, not app-specific shortcuts.

## Windows CE Core OS

- Registry API boundary:
  `/home/royna/WinCE-src_20201004/PRIVATE/WINCEOS/COREOS/NK/KERNEL/fscall.c`
  - `NKRegQueryValueExW` forwards to `FILESYS #25 - RegQueryValueExW`.
  - `NKRegSetValueExW` forwards to `FILESYS #26 - RegSetValueExW`.
  - `NKRegOpenKeyExW` forwards to `FILESYS #23 - RegOpenKeyExW`.
  - `NKRegCloseKey` uses `FSYSAPI_REGCLOSE`.

- Kernel-mode import signatures:
  `/home/royna/WinCE-src_20201004/PRIVATE/WINCEOS/COREOS/INC/kmodeentries.hpp`
  - Defines `NKRegOpenKeyExW_t` and `NKRegQueryValueExW_t` signatures.

- GWE message queue surface:
  `/home/royna/WinCE-src_20201004/PRIVATE/WINCEOS/COREOS/GWE/INC/cmsgque.h`
  - Declares `GetMessageW_I`, `GetMessageWNoWait_I`, `PeekMessageW_I`,
    `PostMessageW_I`, and `SendMessageW_*` queue entry points.

- COREDLL multimedia ordinals:
  `/home/royna/WinCE-src_20201004/PRIVATE/WINCEOS/COREOS/CORE/DLL/core_common.def`
  - Lists waveOut exports including `waveOutSetVolume @382`,
    `waveOutClose @384`, `waveOutWrite @387`, `waveOutReset @390`, and
    `waveOutOpen @399`.

## MFC CE Reference Source

- MFC thread pump:
  `/mnt/c/Program Files (x86)/Microsoft Visual Studio 8/VC/ce/atlmfc/src/mfc/thrdcore.cpp`
  - `AfxInternalPumpMessage` calls `GetMessage`.
  - `CWinThread::Run` uses `PeekMessage(..., PM_NOREMOVE)` for idle detection
    and loops through `PumpMessage`.

- MFC window dispatch:
  `/mnt/c/Program Files (x86)/Microsoft Visual Studio 8/VC/ce/atlmfc/src/mfc/wincore.cpp`
  - `CWnd::WindowProc` calls message-map handling before `DefWindowProc`.
  - Window creation flows through `AfxCtxCreateWindowEx`, `PreCreateWindowEx`,
    and `PostCreateWindowEx`.
