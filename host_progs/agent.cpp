/*
 * agent.cpp - Standalone Windows CE MIPSII remote-control/debug agent.
 *
 * Target:
 *   Userland EXE staged on SD, e.g. \SDMMC Disk\cdx\agent.exe
 *   Transport: TCP listener by default.  This works over ordinary Ethernet,
 *   WiFi, PPP, or ActiveSync pass-through if the OS exposes any of those as
 *   Winsock networking.  The protocol below is transport-agnostic; a later
 *   COM/RAPI shim can frame the same packets.
 *
 * V1 scope:
 *   - PING / INFO
 *   - start process
 *   - kill process
 *   - enumerate processes/modules
 *   - read/write process memory
 *   - read/write files in bounded chunks
 *   - fetch agent log
 *
 * Not in this file yet:
 *   cdxdrv.dll kernel-helper calls, API/IAT tracing, framebuffer snapshots,
 *   touch injection, hooks, exception/register capture, and breakpoints.
 *
 * eVC4/MIPSII build note:
 *   This source is standalone.  Do not add repo build integration yet.
 *   Compile with the Standard SDK 4.2 Mipsii include/lib paths and link with:
 *     coredll.lib winsock.lib toolhelp.lib
 *   The Standard SDK import-lib directory may not contain toolhelp.lib.  In
 *   that case generate a small import lib from the CE source DEF:
 *     lib.exe /machine:MIPS /def:C:\WINCE600\PUBLIC\COMMON\OAK\LIB\MIPSII\RETAIL\toolhelp.def /out:toolhelp_import.lib
 *
 * Wire protocol:
 *   Request header, optional payload:
 *     CdxAgentRequest
 *   Response header, optional payload:
 *     CdxAgentResponse
 *
 *   magic   = 0x31415843  // "CXA1" little endian
 *   version = 1
 *   max payload = 4096 bytes
 *
 * Command payloads:
 *   PING, INFO:
 *     no payload
 *
 *   START_PROCESS:
 *     two null-terminated UTF-16 strings:
 *       image_path\0command_line\0
 *     response.value = process handle value returned by CreateProcessW
 *     response.aux   = thread handle value returned by CreateProcessW
 *
 *   KILL_PROCESS:
 *     request.pid   = target process id
 *     request.value = exit code
 *
 *   ENUM_PROCESSES:
 *     request.value = number of records to skip
 *     response payload = CdxAgentProcessRecord[]
 *     response.value = records returned
 *     response.aux   = next skip value, or 0 when complete
 *
 *   ENUM_MODULES:
 *     request.pid   = target process id
 *     request.value = number of records to skip
 *     response payload = CdxAgentModuleRecord[]
 *
 *   READ_PROCESS_MEMORY:
 *     request.pid/address/length identify the target.
 *     response payload = bytes read
 *
 *   WRITE_PROCESS_MEMORY:
 *     request.pid/address/length identify the target.
 *     request payload = bytes to write
 *
 *   READ_FILE:
 *     request.address = file offset
 *     request.length  = bytes to read, capped to 4096
 *     request payload = null-terminated UTF-16 path
 *     response payload = bytes read
 *
 *   WRITE_FILE:
 *     request.address = file offset
 *     request.length  = bytes to write
 *     request payload = null-terminated UTF-16 path, then file bytes
 *
 *   GET_LOG:
 *     request.address = file offset in \SDMMC Disk\cdx_agent.log
 *     request.length  = bytes to read
 */

#include <winsock2.h>
#include <windows.h>
#include <tlhelp32.h>

#pragma comment(lib, "winsock.lib")

#ifndef INVALID_FILE_SIZE
#define INVALID_FILE_SIZE ((DWORD)0xFFFFFFFF)
#endif

#define CDXA_MAGIC 0x31415843UL /* "CXA1", little endian */
#define CDXA_VERSION 1UL
#define CDXA_DEFAULT_PORT 9876
#define CDXA_MAX_PAYLOAD 4096UL
#define CDXA_LOG_PATH L"\\SDMMC Disk\\cdx_agent.log"
#define CDXA_NAME_CHARS 64
#define CDXA_PATH_CHARS 128

enum CdxAgentCommand {
    CDXA_CMD_PING = 0,
    CDXA_CMD_INFO = 1,
    CDXA_CMD_START_PROCESS = 2,
    CDXA_CMD_KILL_PROCESS = 3,
    CDXA_CMD_ENUM_PROCESSES = 4,
    CDXA_CMD_ENUM_MODULES = 5,
    CDXA_CMD_READ_PROCESS_MEMORY = 6,
    CDXA_CMD_WRITE_PROCESS_MEMORY = 7,
    CDXA_CMD_READ_FILE = 8,
    CDXA_CMD_WRITE_FILE = 9,
    CDXA_CMD_GET_LOG = 10,
    CDXA_CMD_QUIT = 11
};

struct CdxAgentRequest {
    DWORD magic;
    DWORD version;
    DWORD command;
    DWORD sequence;
    DWORD flags;
    DWORD pid;
    DWORD address;
    DWORD length;
    DWORD value;
    DWORD aux;
    DWORD payloadBytes;
    DWORD crc32;
};

struct CdxAgentResponse {
    DWORD magic;
    DWORD version;
    DWORD command;
    DWORD sequence;
    DWORD status;
    DWORD flags;
    DWORD actual;
    DWORD value;
    DWORD aux;
    DWORD payloadBytes;
    DWORD crc32;
};

struct CdxAgentProcessRecord {
    DWORD pid;
    DWORD parentPid;
    DWORD threads;
    DWORD baseAddress;
    DWORD accessKey;
    WCHAR exe[CDXA_NAME_CHARS];
};

struct CdxAgentModuleRecord {
    DWORD moduleId;
    DWORD pid;
    DWORD baseAddress;
    DWORD baseSize;
    DWORD hModule;
    WCHAR module[CDXA_NAME_CHARS];
    WCHAR path[CDXA_PATH_CHARS];
};

static BYTE g_payload[CDXA_MAX_PAYLOAD];
static BYTE g_output[CDXA_MAX_PAYLOAD];

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

static DWORD CdxWideLenBounded(const WCHAR* text, DWORD maxChars) {
    DWORD i;
    if (text == 0) {
        return 0;
    }
    for (i = 0; i < maxChars; ++i) {
        if (text[i] == 0) {
            return i;
        }
    }
    return maxChars;
}

static DWORD CdxAtoiW(const WCHAR* text) {
    DWORD value = 0;
    while (*text >= L'0' && *text <= L'9') {
        value = (value * 10) + (DWORD)(*text - L'0');
        ++text;
    }
    return value;
}

static void CdxLogText(const WCHAR* text) {
    HANDLE file;
    DWORD written = 0;
    SYSTEMTIME st;
    WCHAR line[384];

    GetLocalTime(&st);
    wsprintfW(
        line,
        L"%04u-%02u-%02u %02u:%02u:%02u %s\r\n",
        st.wYear,
        st.wMonth,
        st.wDay,
        st.wHour,
        st.wMinute,
        st.wSecond,
        text);

    file = CreateFileW(CDXA_LOG_PATH, GENERIC_WRITE, 0, 0, OPEN_ALWAYS, FILE_ATTRIBUTE_NORMAL, 0);
    if (file == INVALID_HANDLE_VALUE) {
        return;
    }
    SetFilePointer(file, 0, 0, FILE_END);
    WriteFile(file, line, lstrlenW(line) * sizeof(WCHAR), &written, 0);
    CloseHandle(file);
}

static BOOL CdxRecvAll(SOCKET socketHandle, BYTE* buffer, DWORD bytes) {
    DWORD done = 0;
    while (done < bytes) {
        int got = recv(socketHandle, (char*)buffer + done, (int)(bytes - done), 0);
        if (got <= 0) {
            return FALSE;
        }
        done += (DWORD)got;
    }
    return TRUE;
}

static BOOL CdxSendAll(SOCKET socketHandle, const BYTE* buffer, DWORD bytes) {
    DWORD done = 0;
    while (done < bytes) {
        int sent = send(socketHandle, (const char*)buffer + done, (int)(bytes - done), 0);
        if (sent <= 0) {
            return FALSE;
        }
        done += (DWORD)sent;
    }
    return TRUE;
}

static void CdxInitResponse(CdxAgentResponse* response, const CdxAgentRequest* request) {
    response->magic = CDXA_MAGIC;
    response->version = CDXA_VERSION;
    response->command = request->command;
    response->sequence = request->sequence;
    response->status = ERROR_SUCCESS;
    response->flags = 0;
    response->actual = 0;
    response->value = 0;
    response->aux = 0;
    response->payloadBytes = 0;
    response->crc32 = 0;
}

static BOOL CdxSendResponse(SOCKET socketHandle, CdxAgentResponse* response, const BYTE* payload) {
    DWORD payloadBytes = response->payloadBytes;
    response->crc32 = 0;
    response->crc32 = CdxCrc32((const BYTE*)response, sizeof(CdxAgentResponse));
    if (payloadBytes != 0) {
        response->crc32 ^= CdxCrc32(payload, payloadBytes);
    }
    if (!CdxSendAll(socketHandle, (const BYTE*)response, sizeof(CdxAgentResponse))) {
        return FALSE;
    }
    if (payloadBytes != 0 && !CdxSendAll(socketHandle, payload, payloadBytes)) {
        return FALSE;
    }
    return TRUE;
}

static BOOL CdxPayloadHasWideString(const BYTE* payload, DWORD payloadBytes, const WCHAR** text, DWORD* textBytes) {
    const WCHAR* wide = (const WCHAR*)payload;
    DWORD chars;

    if (payload == 0 || payloadBytes < sizeof(WCHAR) || (payloadBytes & 1) != 0) {
        return FALSE;
    }

    chars = CdxWideLenBounded(wide, payloadBytes / sizeof(WCHAR));
    if (chars == payloadBytes / sizeof(WCHAR)) {
        return FALSE;
    }

    *text = wide;
    *textBytes = (chars + 1) * sizeof(WCHAR);
    return TRUE;
}

static DWORD CdxCommandPing(CdxAgentResponse* response) {
    response->value = GetTickCount();
    response->aux = CDXA_MAX_PAYLOAD;
    return ERROR_SUCCESS;
}

static DWORD CdxCommandInfo(CdxAgentResponse* response) {
    DWORD processId = GetCurrentProcessId();
    DWORD tick = GetTickCount();

    response->flags = 1; /* TCP transport */
    response->actual = sizeof(CdxAgentResponse);
    response->value = processId;
    response->aux = tick;
    return ERROR_SUCCESS;
}

static DWORD CdxCommandStartProcess(const CdxAgentRequest* request, CdxAgentResponse* response, const BYTE* payload) {
    PROCESS_INFORMATION pi;
    const WCHAR* image;
    const WCHAR* commandLine;
    DWORD imageChars;
    DWORD remainingChars;

    if ((request->payloadBytes & 1) != 0 || request->payloadBytes < 2 * sizeof(WCHAR)) {
        return ERROR_INVALID_PARAMETER;
    }

    image = (const WCHAR*)payload;
    imageChars = CdxWideLenBounded(image, request->payloadBytes / sizeof(WCHAR));
    if (imageChars == request->payloadBytes / sizeof(WCHAR)) {
        return ERROR_INVALID_PARAMETER;
    }

    commandLine = image + imageChars + 1;
    remainingChars = (request->payloadBytes / sizeof(WCHAR)) - imageChars - 1;
    if (CdxWideLenBounded(commandLine, remainingChars) == remainingChars) {
        return ERROR_INVALID_PARAMETER;
    }

    ZeroMemory(&pi, sizeof(pi));
    if (!CreateProcessW(image, commandLine[0] ? commandLine : 0, 0, 0, FALSE, 0, 0, 0, 0, &pi)) {
        return GetLastError();
    }

    response->value = (DWORD)pi.hProcess;
    response->aux = (DWORD)pi.hThread;
    if (pi.hThread != 0) {
        CloseHandle(pi.hThread);
    }
    if (pi.hProcess != 0) {
        CloseHandle(pi.hProcess);
    }
    CdxLogText(L"START_PROCESS ok");
    return ERROR_SUCCESS;
}

static DWORD CdxCommandKillProcess(const CdxAgentRequest* request) {
    HANDLE process;
    BOOL ok;

    process = OpenProcess(PROCESS_TERMINATE, FALSE, request->pid);
    if (process == 0) {
        return GetLastError();
    }

    ok = TerminateProcess(process, request->value);
    CloseHandle(process);
    if (!ok) {
        return GetLastError();
    }
    CdxLogText(L"KILL_PROCESS ok");
    return ERROR_SUCCESS;
}

static void CdxCopyWide(WCHAR* destination, DWORD destinationChars, const WCHAR* source) {
    DWORD i;
    if (destinationChars == 0) {
        return;
    }
    for (i = 0; i + 1 < destinationChars && source[i] != 0; ++i) {
        destination[i] = source[i];
    }
    destination[i] = 0;
}

static DWORD CdxCommandEnumProcesses(const CdxAgentRequest* request, CdxAgentResponse* response, BYTE* output) {
    HANDLE snapshot;
    PROCESSENTRY32 pe;
    DWORD skipped = 0;
    DWORD returned = 0;
    DWORD capacity = CDXA_MAX_PAYLOAD / sizeof(CdxAgentProcessRecord);
    BOOL ok;

    snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);
    if (snapshot == INVALID_HANDLE_VALUE) {
        return GetLastError();
    }

    ZeroMemory(&pe, sizeof(pe));
    pe.dwSize = sizeof(pe);
    ok = Process32First(snapshot, &pe);
    while (ok && returned < capacity) {
        if (skipped >= request->value) {
            CdxAgentProcessRecord* record = ((CdxAgentProcessRecord*)output) + returned;
            ZeroMemory(record, sizeof(*record));
            record->pid = pe.th32ProcessID;
            record->parentPid = pe.th32ParentProcessID;
            record->threads = pe.cntThreads;
            record->baseAddress = pe.th32MemoryBase;
            record->accessKey = pe.th32AccessKey;
            CdxCopyWide(record->exe, CDXA_NAME_CHARS, pe.szExeFile);
            ++returned;
        }
        ++skipped;
        ok = Process32Next(snapshot, &pe);
    }

    CloseHandle(snapshot);
    response->payloadBytes = returned * sizeof(CdxAgentProcessRecord);
    response->actual = response->payloadBytes;
    response->value = returned;
    response->aux = ok ? skipped : 0;
    return ERROR_SUCCESS;
}

static DWORD CdxCommandEnumModules(const CdxAgentRequest* request, CdxAgentResponse* response, BYTE* output) {
    HANDLE snapshot;
    MODULEENTRY32 me;
    DWORD skipped = 0;
    DWORD returned = 0;
    DWORD capacity = CDXA_MAX_PAYLOAD / sizeof(CdxAgentModuleRecord);
    BOOL ok;

    snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPMODULE, request->pid);
    if (snapshot == INVALID_HANDLE_VALUE) {
        return GetLastError();
    }

    ZeroMemory(&me, sizeof(me));
    me.dwSize = sizeof(me);
    ok = Module32First(snapshot, &me);
    while (ok && returned < capacity) {
        if (skipped >= request->value) {
            CdxAgentModuleRecord* record = ((CdxAgentModuleRecord*)output) + returned;
            ZeroMemory(record, sizeof(*record));
            record->moduleId = me.th32ModuleID;
            record->pid = me.th32ProcessID;
            record->baseAddress = (DWORD)me.modBaseAddr;
            record->baseSize = me.modBaseSize;
            record->hModule = (DWORD)me.hModule;
            CdxCopyWide(record->module, CDXA_NAME_CHARS, me.szModule);
            CdxCopyWide(record->path, CDXA_PATH_CHARS, me.szExePath);
            ++returned;
        }
        ++skipped;
        ok = Module32Next(snapshot, &me);
    }

    CloseHandle(snapshot);
    response->payloadBytes = returned * sizeof(CdxAgentModuleRecord);
    response->actual = response->payloadBytes;
    response->value = returned;
    response->aux = ok ? skipped : 0;
    return ERROR_SUCCESS;
}

static DWORD CdxCommandReadProcessMemory(const CdxAgentRequest* request, CdxAgentResponse* response, BYTE* output) {
    HANDLE process;
    DWORD bytesRead = 0;
    BOOL ok;

    if (request->length > CDXA_MAX_PAYLOAD) {
        return ERROR_INVALID_PARAMETER;
    }

    process = OpenProcess(PROCESS_VM_READ, FALSE, request->pid);
    if (process == 0) {
        return GetLastError();
    }

    ok = ReadProcessMemory(process, (LPCVOID)request->address, output, request->length, &bytesRead);
    CloseHandle(process);
    if (!ok) {
        return GetLastError();
    }

    response->actual = bytesRead;
    response->payloadBytes = bytesRead;
    return ERROR_SUCCESS;
}

static DWORD CdxCommandWriteProcessMemory(const CdxAgentRequest* request, CdxAgentResponse* response, const BYTE* payload) {
    HANDLE process;
    DWORD bytesWritten = 0;
    BOOL ok;

    if (request->length > CDXA_MAX_PAYLOAD || request->payloadBytes < request->length) {
        return ERROR_INVALID_PARAMETER;
    }

    process = OpenProcess(PROCESS_VM_OPERATION | PROCESS_VM_WRITE, FALSE, request->pid);
    if (process == 0) {
        return GetLastError();
    }

    ok = WriteProcessMemory(process, (LPVOID)request->address, (LPVOID)payload, request->length, &bytesWritten);
    if (ok) {
        FlushInstructionCache(process, (LPCVOID)request->address, bytesWritten);
    }
    CloseHandle(process);
    if (!ok) {
        return GetLastError();
    }

    response->actual = bytesWritten;
    return ERROR_SUCCESS;
}

static DWORD CdxReadFileChunk(const WCHAR* path, DWORD offset, DWORD requested, CdxAgentResponse* response, BYTE* output) {
    HANDLE file;
    DWORD read = 0;
    BOOL ok;

    if (requested > CDXA_MAX_PAYLOAD) {
        return ERROR_INVALID_PARAMETER;
    }

    file = CreateFileW(path, GENERIC_READ, FILE_SHARE_READ, 0, OPEN_EXISTING, FILE_ATTRIBUTE_NORMAL, 0);
    if (file == INVALID_HANDLE_VALUE) {
        return GetLastError();
    }

    if (SetFilePointer(file, offset, 0, FILE_BEGIN) == INVALID_FILE_SIZE && GetLastError() != ERROR_SUCCESS) {
        DWORD error = GetLastError();
        CloseHandle(file);
        return error;
    }

    ok = ReadFile(file, output, requested, &read, 0);
    CloseHandle(file);
    if (!ok) {
        return GetLastError();
    }

    response->actual = read;
    response->payloadBytes = read;
    return ERROR_SUCCESS;
}

static DWORD CdxCommandReadFile(const CdxAgentRequest* request, CdxAgentResponse* response, const BYTE* payload, BYTE* output) {
    const WCHAR* path;
    DWORD pathBytes;

    if (!CdxPayloadHasWideString(payload, request->payloadBytes, &path, &pathBytes)) {
        return ERROR_INVALID_PARAMETER;
    }
    UNREFERENCED_PARAMETER(pathBytes);
    return CdxReadFileChunk(path, request->address, request->length, response, output);
}

static DWORD CdxCommandWriteFile(const CdxAgentRequest* request, CdxAgentResponse* response, const BYTE* payload) {
    const WCHAR* path;
    DWORD pathBytes;
    const BYTE* fileBytes;
    HANDLE file;
    DWORD written = 0;
    BOOL ok;

    if (!CdxPayloadHasWideString(payload, request->payloadBytes, &path, &pathBytes)) {
        return ERROR_INVALID_PARAMETER;
    }
    if (request->length > CDXA_MAX_PAYLOAD || request->payloadBytes < pathBytes + request->length) {
        return ERROR_INVALID_PARAMETER;
    }

    fileBytes = payload + pathBytes;
    file = CreateFileW(path, GENERIC_WRITE, 0, 0, OPEN_ALWAYS, FILE_ATTRIBUTE_NORMAL, 0);
    if (file == INVALID_HANDLE_VALUE) {
        return GetLastError();
    }
    if (SetFilePointer(file, request->address, 0, FILE_BEGIN) == INVALID_FILE_SIZE && GetLastError() != ERROR_SUCCESS) {
        DWORD error = GetLastError();
        CloseHandle(file);
        return error;
    }

    ok = WriteFile(file, fileBytes, request->length, &written, 0);
    CloseHandle(file);
    if (!ok) {
        return GetLastError();
    }

    response->actual = written;
    return ERROR_SUCCESS;
}

static DWORD CdxCommandGetLog(const CdxAgentRequest* request, CdxAgentResponse* response, BYTE* output) {
    return CdxReadFileChunk(CDXA_LOG_PATH, request->address, request->length, response, output);
}

static BOOL CdxHandleClient(SOCKET clientSocket) {
    CdxAgentRequest request;
    CdxAgentResponse response;
    BOOL keepGoing = TRUE;

    CdxLogText(L"client connected");

    while (keepGoing) {
        DWORD status = ERROR_SUCCESS;

        if (!CdxRecvAll(clientSocket, (BYTE*)&request, sizeof(request))) {
            break;
        }

        if (request.magic != CDXA_MAGIC || request.version != CDXA_VERSION || request.payloadBytes > CDXA_MAX_PAYLOAD) {
            CdxAgentRequest synthetic;
            ZeroMemory(&synthetic, sizeof(synthetic));
            synthetic.command = request.command;
            synthetic.sequence = request.sequence;
            CdxInitResponse(&response, &synthetic);
            response.status = ERROR_INVALID_PARAMETER;
            CdxSendResponse(clientSocket, &response, 0);
            break;
        }

        if (request.payloadBytes != 0 && !CdxRecvAll(clientSocket, g_payload, request.payloadBytes)) {
            break;
        }

        CdxInitResponse(&response, &request);
        ZeroMemory(g_output, sizeof(g_output));

        switch (request.command) {
        case CDXA_CMD_PING:
            status = CdxCommandPing(&response);
            break;
        case CDXA_CMD_INFO:
            status = CdxCommandInfo(&response);
            break;
        case CDXA_CMD_START_PROCESS:
            status = CdxCommandStartProcess(&request, &response, g_payload);
            break;
        case CDXA_CMD_KILL_PROCESS:
            status = CdxCommandKillProcess(&request);
            break;
        case CDXA_CMD_ENUM_PROCESSES:
            status = CdxCommandEnumProcesses(&request, &response, g_output);
            break;
        case CDXA_CMD_ENUM_MODULES:
            status = CdxCommandEnumModules(&request, &response, g_output);
            break;
        case CDXA_CMD_READ_PROCESS_MEMORY:
            status = CdxCommandReadProcessMemory(&request, &response, g_output);
            break;
        case CDXA_CMD_WRITE_PROCESS_MEMORY:
            status = CdxCommandWriteProcessMemory(&request, &response, g_payload);
            break;
        case CDXA_CMD_READ_FILE:
            status = CdxCommandReadFile(&request, &response, g_payload, g_output);
            break;
        case CDXA_CMD_WRITE_FILE:
            status = CdxCommandWriteFile(&request, &response, g_payload);
            break;
        case CDXA_CMD_GET_LOG:
            status = CdxCommandGetLog(&request, &response, g_output);
            break;
        case CDXA_CMD_QUIT:
            status = ERROR_SUCCESS;
            keepGoing = FALSE;
            break;
        default:
            status = ERROR_INVALID_PARAMETER;
            break;
        }

        response.status = status;
        if (!CdxSendResponse(clientSocket, &response, response.payloadBytes ? g_output : 0)) {
            break;
        }
    }

    CdxLogText(L"client disconnected");
    return TRUE;
}

static WORD CdxParsePort(LPWSTR commandLine) {
    WCHAR* p = commandLine;
    while (p != 0 && *p != 0) {
        while (*p == L' ' || *p == L'\t') {
            ++p;
        }
        if ((p[0] == L'/' || p[0] == L'-') &&
            (p[1] == L'p' || p[1] == L'P') &&
            (p[2] == L'o' || p[2] == L'O') &&
            (p[3] == L'r' || p[3] == L'R') &&
            (p[4] == L't' || p[4] == L'T') &&
            p[5] == L':') {
            DWORD port = CdxAtoiW(p + 6);
            if (port > 0 && port < 65536) {
                return (WORD)port;
            }
        }
        while (*p != 0 && *p != L' ' && *p != L'\t') {
            ++p;
        }
    }
    return (WORD)CDXA_DEFAULT_PORT;
}

static int CdxRunServer(WORD port) {
    WSADATA wsa;
    SOCKET listenSocket;
    sockaddr_in address;
    WCHAR logLine[128];

    if (WSAStartup(MAKEWORD(2, 0), &wsa) != 0) {
        return 1;
    }

    listenSocket = socket(AF_INET, SOCK_STREAM, IPPROTO_TCP);
    if (listenSocket == INVALID_SOCKET) {
        WSACleanup();
        return 2;
    }

    ZeroMemory(&address, sizeof(address));
    address.sin_family = AF_INET;
    address.sin_port = htons(port);
    address.sin_addr.s_addr = INADDR_ANY;

    if (bind(listenSocket, (sockaddr*)&address, sizeof(address)) == SOCKET_ERROR) {
        closesocket(listenSocket);
        WSACleanup();
        return 3;
    }
    if (listen(listenSocket, 1) == SOCKET_ERROR) {
        closesocket(listenSocket);
        WSACleanup();
        return 4;
    }

    wsprintfW(logLine, L"agent listening on tcp port %u", port);
    CdxLogText(logLine);

    for (;;) {
        SOCKET client = accept(listenSocket, 0, 0);
        if (client == INVALID_SOCKET) {
            break;
        }
        CdxHandleClient(client);
        closesocket(client);
    }

    closesocket(listenSocket);
    WSACleanup();
    return 0;
}

int WINAPI WinMain(HINSTANCE instance, HINSTANCE previous, LPWSTR commandLine, int showCommand) {
    WORD port;
    UNREFERENCED_PARAMETER(instance);
    UNREFERENCED_PARAMETER(previous);
    UNREFERENCED_PARAMETER(showCommand);

    port = CdxParsePort(commandLine ? commandLine : L"");
    CdxLogText(L"agent starting");
    return CdxRunServer(port);
}
