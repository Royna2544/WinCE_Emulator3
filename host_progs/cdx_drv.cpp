/*
 * cdx_drv.cpp - Standalone Windows CE MIPSII debug/memory-core stream driver.
 *
 * Target:
 *   DLL name: CDXDBG.dll
 *   Device:   CDX1:
 *   Loader:   ActivateDeviceEx/RegisterDevice-style userland agent
 *   Client:   CreateFile(L"CDX1:", ...) + DeviceIoControl(...)
 *
 * V1 scope:
 *   Memory core only.  Screen mirror, touch injection, API tracing, and
 *   breakpoints are intentionally not implemented here.
 *
 * eVC4/MIPSII build note:
 *   This source is deliberately standalone.  Do not add repo build integration
 *   yet.  Build it with the CE/eVC4 MIPSII compiler and SDK headers/libs, e.g.
 *   the Standard SDK 4.2 Mipsii include/lib directories documented in
 *   tests/test_progs/build_notes/evc4_mipsii.md, and link as a DLL.  If your
 *   linker ignores #pragma comment(linker, ...), pass:
 *     /entry:DllMain /export:DllMain
 *   The CDX_* entry points are exported through __declspec(dllexport).
 *
 * Protocol:
 *   All controls use METHOD_BUFFERED CTL_CODE values from winioctl.h:
 *     CDX_IOCTL(fn) = CTL_CODE(0x8337, 0x800 + fn, METHOD_BUFFERED, FILE_ANY_ACCESS)
 *
 *   Input starts with CdxRequest.  WRITE_VIRT payload bytes follow the request.
 *   Output starts with CdxResponse.  READ_VIRT bytes follow the response.
 *   crc32 in responses is filled over the returned bytes with crc32 set to 0.
 *
 * Safety model:
 *   QUERY, READ, PROBE, and OAL_GET_MASK work while locked.
 *   WRITE, COPY, FILL, WRITE_SCALAR, and OAL_SET_MASK require per-open unlock.
 *   Unlock key is nonce ^ 0xC0DEC0DE, where nonce is returned by QUERY.
 *   Raw virtual memory access is bounded to 4096 bytes and wrapped in SEH.
 */

#include <windows.h>
#include <winioctl.h>

#pragma comment(linker, "/export:DllMain")

#ifndef ERROR_NOT_SUPPORTED
#define ERROR_NOT_SUPPORTED 50L
#endif

#define CDX_MAGIC 0x30584443UL /* "CDX0", little endian */
#define CDX_VERSION 1UL
#define CDX_UNLOCK_XOR 0xC0DEC0DEUL
#define CDX_MAX_TRANSFER 4096UL
#define CDX_FILE_DEVICE 0x8337UL
#define CDX_IOCTL(fn) CTL_CODE(CDX_FILE_DEVICE, 0x800 + (fn), METHOD_BUFFERED, FILE_ANY_ACCESS)

enum CdxIoctlFn {
    CDX_FN_QUERY = 0,
    CDX_FN_UNLOCK = 1,
    CDX_FN_LOCK = 2,
    CDX_FN_READ_VIRT = 3,
    CDX_FN_WRITE_VIRT = 4,
    CDX_FN_COPY_VIRT = 5,
    CDX_FN_FILL_VIRT = 6,
    CDX_FN_READ_SCALAR = 7,
    CDX_FN_WRITE_SCALAR = 8,
    CDX_FN_PROBE_VIRT = 9,
    CDX_FN_OAL_GET_MASK = 10,
    CDX_FN_OAL_SET_MASK = 11,
    CDX_FN_MAP_PHYS_WINDOW = 12,
    CDX_FN_UNMAP_PHYS_WINDOW = 13
};

#define IOCTL_CDX_QUERY CDX_IOCTL(CDX_FN_QUERY)
#define IOCTL_CDX_UNLOCK CDX_IOCTL(CDX_FN_UNLOCK)
#define IOCTL_CDX_LOCK CDX_IOCTL(CDX_FN_LOCK)
#define IOCTL_CDX_READ_VIRT CDX_IOCTL(CDX_FN_READ_VIRT)
#define IOCTL_CDX_WRITE_VIRT CDX_IOCTL(CDX_FN_WRITE_VIRT)
#define IOCTL_CDX_COPY_VIRT CDX_IOCTL(CDX_FN_COPY_VIRT)
#define IOCTL_CDX_FILL_VIRT CDX_IOCTL(CDX_FN_FILL_VIRT)
#define IOCTL_CDX_READ_SCALAR CDX_IOCTL(CDX_FN_READ_SCALAR)
#define IOCTL_CDX_WRITE_SCALAR CDX_IOCTL(CDX_FN_WRITE_SCALAR)
#define IOCTL_CDX_PROBE_VIRT CDX_IOCTL(CDX_FN_PROBE_VIRT)
#define IOCTL_CDX_OAL_GET_MASK CDX_IOCTL(CDX_FN_OAL_GET_MASK)
#define IOCTL_CDX_OAL_SET_MASK CDX_IOCTL(CDX_FN_OAL_SET_MASK)
#define IOCTL_CDX_MAP_PHYS_WINDOW CDX_IOCTL(CDX_FN_MAP_PHYS_WINDOW)
#define IOCTL_CDX_UNMAP_PHYS_WINDOW CDX_IOCTL(CDX_FN_UNMAP_PHYS_WINDOW)

struct CdxRequest {
    DWORD magic;
    DWORD version;
    DWORD session;
    DWORD flags;
    DWORD address;
    DWORD length;
    DWORD value;
    DWORD aux;
    DWORD crc32;
};

struct CdxResponse {
    DWORD magic;
    DWORD version;
    DWORD status;
    DWORD flags;
    DWORD actual;
    DWORD value;
    DWORD aux;
    DWORD crc32;
};

struct CdxDeviceContext {
    DWORD signature;
};

struct CdxOpenContext {
    DWORD signature;
    DWORD session;
    DWORD nonce;
    BOOL unlocked;
    CdxDeviceContext* device;
};

#define CDX_DEVICE_SIGNATURE 0x44584344UL /* "DCXD" */
#define CDX_OPEN_SIGNATURE 0x4f584344UL   /* "DCXO" */

/* Observed OAL/debug code lives around AFC86E54; this range scans the resident
 * OAL area without hardcoding the DBGPARAM table or mask address.
 * Callers may override with request.address/request.length for GET/SET_MASK.
 */
#define CDX_OAL_SCAN_BASE 0xAFC00000UL
#define CDX_OAL_SCAN_LENGTH 0x00400000UL
#define CDX_DBGPARAM_NAME_CHARS 32
#define CDX_DBGPARAM_ZONE_COUNT 16
#define CDX_DBGPARAM_ZONE_CHARS 32
#define CDX_DBGPARAM_MASK_OFFSET \
    ((CDX_DBGPARAM_NAME_CHARS + (CDX_DBGPARAM_ZONE_COUNT * CDX_DBGPARAM_ZONE_CHARS)) * sizeof(WCHAR))

static LONG g_nextSession = 0;

static DWORD CdxCrc32(const BYTE* data, DWORD length) {
    DWORD crc = 0xFFFFFFFFUL;
    DWORD i;
    int bit;

    for (i = 0; i < length; ++i) {
        crc ^= data[i];
        for (bit = 0; bit < 8; ++bit) {
            if (crc & 1) {
                crc = (crc >> 1) ^ 0xEDB88320UL;
            } else {
                crc >>= 1;
            }
        }
    }

    return ~crc;
}

static void CdxInitResponse(CdxResponse* response, DWORD session) {
    response->magic = CDX_MAGIC;
    response->version = CDX_VERSION;
    response->status = ERROR_SUCCESS;
    response->flags = 0;
    response->actual = 0;
    response->value = 0;
    response->aux = session;
    response->crc32 = 0;
}

static void CdxFinalizeResponse(CdxResponse* response, PBYTE output, DWORD totalBytes) {
    response->crc32 = 0;
    response->crc32 = CdxCrc32(output, totalBytes);
}

static BOOL CdxCheckedAdd(DWORD a, DWORD b, DWORD* result) {
    DWORD value = a + b;
    if (value < a) {
        return FALSE;
    }
    *result = value;
    return TRUE;
}

static BOOL CdxSafeRead(void* destination, const void* source, DWORD length) {
    DWORD i;
    volatile const BYTE* src = (volatile const BYTE*)source;
    BYTE* dst = (BYTE*)destination;

    __try {
        for (i = 0; i < length; ++i) {
            dst[i] = src[i];
        }
    } __except (EXCEPTION_EXECUTE_HANDLER) {
        return FALSE;
    }

    return TRUE;
}

static BOOL CdxSafeWrite(void* destination, const void* source, DWORD length) {
    DWORD i;
    volatile BYTE* dst = (volatile BYTE*)destination;
    const BYTE* src = (const BYTE*)source;

    __try {
        for (i = 0; i < length; ++i) {
            dst[i] = src[i];
        }
    } __except (EXCEPTION_EXECUTE_HANDLER) {
        return FALSE;
    }

    return TRUE;
}

static BOOL CdxSafeFill(void* destination, BYTE value, DWORD length) {
    DWORD i;
    volatile BYTE* dst = (volatile BYTE*)destination;

    __try {
        for (i = 0; i < length; ++i) {
            dst[i] = value;
        }
    } __except (EXCEPTION_EXECUTE_HANDLER) {
        return FALSE;
    }

    return TRUE;
}

static BOOL CdxSafeReadScalar(DWORD address, DWORD width, DWORD* value) {
    __try {
        if (width == 1) {
            *value = *(volatile BYTE*)address;
        } else if (width == 2) {
            *value = *(volatile WORD*)address;
        } else if (width == 4) {
            *value = *(volatile DWORD*)address;
        } else {
            return FALSE;
        }
    } __except (EXCEPTION_EXECUTE_HANDLER) {
        return FALSE;
    }

    return TRUE;
}

static BOOL CdxSafeWriteScalar(DWORD address, DWORD width, DWORD value) {
    __try {
        if (width == 1) {
            *(volatile BYTE*)address = (BYTE)value;
        } else if (width == 2) {
            *(volatile WORD*)address = (WORD)value;
        } else if (width == 4) {
            *(volatile DWORD*)address = value;
        } else {
            return FALSE;
        }
    } __except (EXCEPTION_EXECUTE_HANDLER) {
        return FALSE;
    }

    return TRUE;
}

static BOOL CdxSafeProbe(DWORD address, DWORD length) {
    DWORD i;
    volatile BYTE sink;
    volatile const BYTE* src = (volatile const BYTE*)address;

    __try {
        for (i = 0; i < length; ++i) {
            sink = src[i];
        }
    } __except (EXCEPTION_EXECUTE_HANDLER) {
        return FALSE;
    }

    return TRUE;
}

static BOOL CdxSafeWideEquals(DWORD address, const WCHAR* text) {
    DWORD i;

    __try {
        for (i = 0; i < CDX_DBGPARAM_ZONE_CHARS; ++i) {
            WCHAR got = ((volatile const WCHAR*)address)[i];
            WCHAR want = text[i];
            if (got != want) {
                return FALSE;
            }
            if (want == 0) {
                return TRUE;
            }
        }
    } __except (EXCEPTION_EXECUTE_HANDLER) {
        return FALSE;
    }

    return FALSE;
}

static BOOL CdxLooksLikeOalDbgParamNameAt(DWORD nameAddress) {
    DWORD zoneBase = nameAddress + (CDX_DBGPARAM_NAME_CHARS * sizeof(WCHAR));

    if (!CdxSafeWideEquals(nameAddress, L"OAL")) {
        return FALSE;
    }
    if (!CdxSafeWideEquals(zoneBase + (0 * CDX_DBGPARAM_ZONE_CHARS * sizeof(WCHAR)), L"Error")) {
        return FALSE;
    }
    if (!CdxSafeWideEquals(zoneBase + (1 * CDX_DBGPARAM_ZONE_CHARS * sizeof(WCHAR)), L"Warning")) {
        return FALSE;
    }
    if (!CdxSafeWideEquals(zoneBase + (2 * CDX_DBGPARAM_ZONE_CHARS * sizeof(WCHAR)), L"Function")) {
        return FALSE;
    }
    if (!CdxSafeWideEquals(zoneBase + (3 * CDX_DBGPARAM_ZONE_CHARS * sizeof(WCHAR)), L"Info")) {
        return FALSE;
    }
    if (!CdxSafeWideEquals(zoneBase + (15 * CDX_DBGPARAM_ZONE_CHARS * sizeof(WCHAR)), L"Verbose")) {
        return FALSE;
    }

    return TRUE;
}

static BOOL CdxFindOalDbgParam(DWORD scanBase, DWORD scanLength, DWORD* nameAddress, DWORD* maskAddress, DWORD* maskValue) {
    DWORD end;
    DWORD p;
    DWORD value;

    if (scanLength == 0 || !CdxCheckedAdd(scanBase, scanLength, &end)) {
        return FALSE;
    }

    for (p = scanBase; p + CDX_DBGPARAM_MASK_OFFSET + sizeof(DWORD) >= p && p + CDX_DBGPARAM_MASK_OFFSET + sizeof(DWORD) <= end; p += sizeof(WCHAR)) {
        if (CdxLooksLikeOalDbgParamNameAt(p)) {
            DWORD candidateMaskAddress = p + CDX_DBGPARAM_MASK_OFFSET;
            if (CdxSafeReadScalar(candidateMaskAddress, sizeof(DWORD), &value)) {
                *nameAddress = p;
                *maskAddress = candidateMaskAddress;
                *maskValue = value;
                return TRUE;
            }
        }
    }

    return FALSE;
}

static BOOL CdxValidOpen(CdxOpenContext* open) {
    return open != 0 && open->signature == CDX_OPEN_SIGNATURE && open->device != 0 &&
           open->device->signature == CDX_DEVICE_SIGNATURE;
}

static DWORD CdxFunctionFromIoctl(DWORD code, BOOL* known) {
    DWORD function = (code >> 2) & 0xFFF;

    if ((code & 0xFFFF0003UL) != ((CDX_FILE_DEVICE << 16) | METHOD_BUFFERED)) {
        *known = FALSE;
        return 0;
    }

    if (function < 0x800 || function > 0x80D) {
        *known = FALSE;
        return 0;
    }

    *known = TRUE;
    return function - 0x800;
}

BOOL WINAPI DllMain(HANDLE hModule, DWORD reason, LPVOID reserved) {
    UNREFERENCED_PARAMETER(hModule);
    UNREFERENCED_PARAMETER(reason);
    UNREFERENCED_PARAMETER(reserved);
    return TRUE;
}

extern "C" __declspec(dllexport) DWORD CDX_Init(DWORD context) {
    CdxDeviceContext* device;
    UNREFERENCED_PARAMETER(context);

    device = (CdxDeviceContext*)LocalAlloc(LPTR, sizeof(CdxDeviceContext));
    if (device == 0) {
        SetLastError(ERROR_OUTOFMEMORY);
        return 0;
    }

    device->signature = CDX_DEVICE_SIGNATURE;
    return (DWORD)device;
}

extern "C" __declspec(dllexport) BOOL CDX_Deinit(DWORD deviceContext) {
    CdxDeviceContext* device = (CdxDeviceContext*)deviceContext;

    if (device == 0 || device->signature != CDX_DEVICE_SIGNATURE) {
        SetLastError(ERROR_INVALID_HANDLE);
        return FALSE;
    }

    device->signature = 0;
    LocalFree(device);
    return TRUE;
}

extern "C" __declspec(dllexport) DWORD CDX_Open(DWORD deviceContext, DWORD accessCode, DWORD shareMode) {
    CdxDeviceContext* device = (CdxDeviceContext*)deviceContext;
    CdxOpenContext* open;
    DWORD session;

    UNREFERENCED_PARAMETER(accessCode);
    UNREFERENCED_PARAMETER(shareMode);

    if (device == 0 || device->signature != CDX_DEVICE_SIGNATURE) {
        SetLastError(ERROR_INVALID_HANDLE);
        return 0;
    }

    open = (CdxOpenContext*)LocalAlloc(LPTR, sizeof(CdxOpenContext));
    if (open == 0) {
        SetLastError(ERROR_OUTOFMEMORY);
        return 0;
    }

    session = (DWORD)InterlockedIncrement(&g_nextSession);
    open->signature = CDX_OPEN_SIGNATURE;
    open->session = session;
    open->nonce = GetTickCount() ^ session ^ (DWORD)open;
    open->unlocked = FALSE;
    open->device = device;
    return (DWORD)open;
}

extern "C" __declspec(dllexport) BOOL CDX_Close(DWORD openContext) {
    CdxOpenContext* open = (CdxOpenContext*)openContext;

    if (!CdxValidOpen(open)) {
        SetLastError(ERROR_INVALID_HANDLE);
        return FALSE;
    }

    open->signature = 0;
    LocalFree(open);
    return TRUE;
}

extern "C" __declspec(dllexport) DWORD CDX_Read(DWORD openContext, LPVOID buffer, DWORD count) {
    UNREFERENCED_PARAMETER(openContext);
    UNREFERENCED_PARAMETER(buffer);
    UNREFERENCED_PARAMETER(count);
    SetLastError(ERROR_NOT_SUPPORTED);
    return (DWORD)-1;
}

extern "C" __declspec(dllexport) DWORD CDX_Write(DWORD openContext, LPCVOID buffer, DWORD count) {
    UNREFERENCED_PARAMETER(openContext);
    UNREFERENCED_PARAMETER(buffer);
    UNREFERENCED_PARAMETER(count);
    SetLastError(ERROR_NOT_SUPPORTED);
    return (DWORD)-1;
}

extern "C" __declspec(dllexport) DWORD CDX_Seek(DWORD openContext, LONG amount, DWORD type) {
    UNREFERENCED_PARAMETER(openContext);
    UNREFERENCED_PARAMETER(amount);
    UNREFERENCED_PARAMETER(type);
    SetLastError(ERROR_NOT_SUPPORTED);
    return (DWORD)-1;
}

extern "C" __declspec(dllexport) BOOL CDX_IOControl(
    DWORD openContext,
    DWORD code,
    PBYTE input,
    DWORD inputLength,
    PBYTE output,
    DWORD outputLength,
    PDWORD actualOutput) {
    CdxOpenContext* open = (CdxOpenContext*)openContext;
    CdxRequest* request = (CdxRequest*)input;
    CdxResponse* response = (CdxResponse*)output;
    DWORD status = ERROR_SUCCESS;
    DWORD fn;
    DWORD totalOut = sizeof(CdxResponse);
    BOOL known;

    if (actualOutput != 0) {
        *actualOutput = 0;
    }

    if (!CdxValidOpen(open)) {
        SetLastError(ERROR_INVALID_HANDLE);
        return FALSE;
    }

    fn = CdxFunctionFromIoctl(code, &known);
    if (!known) {
        SetLastError(ERROR_INVALID_PARAMETER);
        return FALSE;
    }

    if (input == 0 || inputLength < sizeof(CdxRequest) || output == 0 || outputLength < sizeof(CdxResponse)) {
        SetLastError(ERROR_INVALID_PARAMETER);
        return FALSE;
    }

    CdxInitResponse(response, open->session);

    if (request->magic != CDX_MAGIC || request->version != CDX_VERSION) {
        status = ERROR_INVALID_PARAMETER;
        goto done;
    }

    if (request->session != 0 && request->session != open->session) {
        status = ERROR_INVALID_PARAMETER;
        goto done;
    }

    if (request->length > CDX_MAX_TRANSFER) {
        status = ERROR_INVALID_PARAMETER;
        goto done;
    }

    switch (fn) {
    case CDX_FN_QUERY:
        response->flags = open->unlocked ? 1 : 0;
        response->actual = sizeof(CdxResponse);
        response->value = open->nonce;
        response->aux = open->session;
        break;

    case CDX_FN_UNLOCK:
        if (request->value == (open->nonce ^ CDX_UNLOCK_XOR)) {
            open->unlocked = TRUE;
            response->flags = 1;
            response->value = open->nonce;
        } else {
            status = ERROR_ACCESS_DENIED;
        }
        break;

    case CDX_FN_LOCK:
        open->unlocked = FALSE;
        response->flags = 0;
        response->value = open->nonce;
        break;

    case CDX_FN_READ_VIRT:
        if (outputLength < sizeof(CdxResponse) + request->length) {
            status = ERROR_INSUFFICIENT_BUFFER;
            break;
        }
        if (!CdxSafeRead(output + sizeof(CdxResponse), (const void*)request->address, request->length)) {
            status = ERROR_INVALID_ADDRESS;
            break;
        }
        response->actual = request->length;
        totalOut = sizeof(CdxResponse) + request->length;
        break;

    case CDX_FN_WRITE_VIRT:
        if (!open->unlocked) {
            status = ERROR_ACCESS_DENIED;
            break;
        }
        if (inputLength < sizeof(CdxRequest) + request->length) {
            status = ERROR_INVALID_PARAMETER;
            break;
        }
        if (!CdxSafeWrite((void*)request->address, input + sizeof(CdxRequest), request->length)) {
            status = ERROR_INVALID_ADDRESS;
            break;
        }
        response->actual = request->length;
        break;

    case CDX_FN_COPY_VIRT: {
        BYTE scratch[CDX_MAX_TRANSFER];
        if (!open->unlocked) {
            status = ERROR_ACCESS_DENIED;
            break;
        }
        if (!CdxSafeRead(scratch, (const void*)request->address, request->length) ||
            !CdxSafeWrite((void*)request->aux, scratch, request->length)) {
            status = ERROR_INVALID_ADDRESS;
            break;
        }
        response->actual = request->length;
        break;
    }

    case CDX_FN_FILL_VIRT:
        if (!open->unlocked) {
            status = ERROR_ACCESS_DENIED;
            break;
        }
        if (!CdxSafeFill((void*)request->address, (BYTE)request->value, request->length)) {
            status = ERROR_INVALID_ADDRESS;
            break;
        }
        response->actual = request->length;
        break;

    case CDX_FN_READ_SCALAR:
        if (!CdxSafeReadScalar(request->address, request->length, &response->value)) {
            status = ERROR_INVALID_ADDRESS;
            break;
        }
        response->actual = request->length;
        break;

    case CDX_FN_WRITE_SCALAR:
        if (!open->unlocked) {
            status = ERROR_ACCESS_DENIED;
            break;
        }
        if (!CdxSafeWriteScalar(request->address, request->length, request->value)) {
            status = ERROR_INVALID_ADDRESS;
            break;
        }
        response->actual = request->length;
        break;

    case CDX_FN_PROBE_VIRT:
        if (!CdxSafeProbe(request->address, request->length)) {
            status = ERROR_INVALID_ADDRESS;
            break;
        }
        response->actual = request->length;
        break;

    case CDX_FN_OAL_GET_MASK: {
        DWORD scanBase = request->address ? request->address : CDX_OAL_SCAN_BASE;
        DWORD scanLength = request->length ? request->length : CDX_OAL_SCAN_LENGTH;
        DWORD nameAddress = 0;
        DWORD maskAddress = 0;
        DWORD maskValue = 0;

        if (!CdxFindOalDbgParam(scanBase, scanLength, &nameAddress, &maskAddress, &maskValue)) {
            status = ERROR_NOT_FOUND;
            break;
        }
        response->actual = sizeof(DWORD);
        response->value = maskValue;
        response->aux = maskAddress;
        response->flags = nameAddress;
        break;
    }

    case CDX_FN_OAL_SET_MASK: {
        DWORD scanBase = request->address ? request->address : CDX_OAL_SCAN_BASE;
        DWORD scanLength = request->length ? request->length : CDX_OAL_SCAN_LENGTH;
        DWORD nameAddress = 0;
        DWORD maskAddress = 0;
        DWORD oldMask = 0;

        if (!open->unlocked) {
            status = ERROR_ACCESS_DENIED;
            break;
        }
        if (!CdxFindOalDbgParam(scanBase, scanLength, &nameAddress, &maskAddress, &oldMask)) {
            status = ERROR_NOT_FOUND;
            break;
        }
        if (!CdxSafeWriteScalar(maskAddress, sizeof(DWORD), request->value)) {
            status = ERROR_INVALID_ADDRESS;
            break;
        }
        response->actual = sizeof(DWORD);
        response->value = request->value;
        response->aux = maskAddress;
        response->flags = nameAddress;
        break;
    }

    case CDX_FN_MAP_PHYS_WINDOW:
    case CDX_FN_UNMAP_PHYS_WINDOW:
        status = ERROR_NOT_SUPPORTED;
        break;

    default:
        status = ERROR_INVALID_PARAMETER;
        break;
    }

done:
    response->status = status;
    if (status != ERROR_SUCCESS && response->actual == 0) {
        totalOut = sizeof(CdxResponse);
    }
    CdxFinalizeResponse(response, output, totalOut);
    if (actualOutput != 0) {
        *actualOutput = totalOut;
    }
    SetLastError(status);
    return status == ERROR_SUCCESS;
}
