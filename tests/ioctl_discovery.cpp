/*
 * ioctl_discovery.cpp
 *
 * No-argument Windows CE/eVC4 MIPSII probe for stream-device IOCTL discovery.
 * It opens the known dumped-device names, invokes only read-only/read-like
 * DeviceIoControl contracts, writes a text report to:
 *
 *     \SDMMC Disk\ioctl_discovery.txt
 *
 * and then shows a MessageBoxW summary. Mutating controls are included in the
 * report as SKIP entries so the known-code ledger is complete without changing
 * device state.
 */

#include <windows.h>

#define REPORT_PATH L"\\SDMMC Disk\\ioctl_discovery.txt"
#define REPORT_CAPACITY 32768
#define ARRAY_COUNT(a) (sizeof(a) / sizeof((a)[0]))

struct Probe {
    LPCWSTR device;
    const char* device_name;
    const char* name;
    DWORD code;
    const BYTE* input;
    DWORD input_len;
    DWORD output_len;
    BOOL readonly;
};

static char g_report[REPORT_CAPACITY];
static DWORD g_report_len = 0;

static const BYTE INPUT_ZERO[1] = {0x00};
static const BYTE INPUT_UID_SECTOR[4] = {0x07, 0x00, 0x00, 0x00};
static const BYTE INPUT_PIC_EEPROM_READ[1] = {0x10};
static const BYTE INPUT_MFS_READ[4] = {0x00, 0x40, 0x01, 0x00};
static const BYTE INPUT_I2C_READ[1] = {0x10};

static const Probe PROBES[] = {
    {L"UID1:", "UID1", "NAND_READ_UUID", 0xa00000cc, 0, 0, 4, TRUE},
    {L"UID1:", "UID1", "NAND_READ_UUID_BY_SECTOR", 0xa00000d0, INPUT_UID_SECTOR, sizeof(INPUT_UID_SECTOR), 4, TRUE},
    {L"UID1:", "UID1", "NAND_WRITE_UUID_BY_SECTOR", 0xa000002c, 0, 0, 0, FALSE},
    {L"UID1:", "UID1", "MICOM_RESET", 0x0023d021, 0, 0, 0, FALSE},
    {L"UID1:", "UID1", "TOUCHPAD_LOCK", 0x00221011, 0, 0, 0, FALSE},
    {L"UID1:", "UID1", "TOUCHPAD_UNLOCK", 0x00221015, 0, 0, 0, FALSE},
    {L"UID1:", "UID1", "MICOM_LOCK", 0x00221019, 0, 0, 0, FALSE},
    {L"UID1:", "UID1", "MICOM_UNLOCK", 0x0022101d, 0, 0, 0, FALSE},
    {L"UID1:", "UID1", "NAND_CPU_LOAD_CONTROL", 0x00221025, 0, 0, 0, FALSE},
    {L"UID1:", "UID1", "NAND_CPU_LOAD_CONTROL_ALT", 0x0023c041, 0, 0, 0, FALSE},

    {L"PIC1:", "PIC1", "PIC_READ_VERSION", 0xd0000004, 0, 0, 1, TRUE},
    {L"PIC1:", "PIC1", "PIC_I2C_READ_PIC", 0xd000000c, 0, 0, 0, TRUE},
    {L"PIC1:", "PIC1", "PIC_EEPROM_READ", 0xd0000020, INPUT_PIC_EEPROM_READ, sizeof(INPUT_PIC_EEPROM_READ), 1, TRUE},
    {L"PIC1:", "PIC1", "PIC_I2C_INIT", 0xd0000008, 0, 0, 0, FALSE},
    {L"PIC1:", "PIC1", "PIC_I2C_RESET", 0xd0000010, 0, 0, 0, FALSE},
    {L"PIC1:", "PIC1", "PIC_I2C_SLEEP_OK", 0xd0000014, 0, 0, 0, FALSE},
    {L"PIC1:", "PIC1", "PIC_I2C_PWR_ON_OK", 0xd0000018, 0, 0, 0, FALSE},
    {L"PIC1:", "PIC1", "PIC_EEPROM_WRITE", 0xd000001c, 0, 0, 0, FALSE},
    {L"PIC1:", "PIC1", "PIC_DISPLAY_STATE", 0xd000002c, 0, 0, 0, FALSE},
    {L"PIC1:", "PIC1", "PIC_OS_UPGRADE", 0xd000003c, 0, 0, 0, FALSE},
    {L"PIC1:", "PIC1", "PIC_PWR_LED", 0xd0000040, 0, 0, 0, FALSE},
    {L"PIC1:", "PIC1", "NANDUUID_MICOM_RESET_ACK", 0xa0070014, 0, 0, 0, FALSE},

    {L"LSD1:", "LSD1", "LSD_READ_LUX", 0xd2000008, 0, 0, 2, TRUE},
    {L"LSD1:", "LSD1", "LSD_SET_CONTROL", 0xd2000004, 0, 0, 0, FALSE},
    {L"LSD1:", "LSD1", "LSD_START_SENSING", 0xd2000014, 0, 0, 0, FALSE},
    {L"LSD1:", "LSD1", "LSD_STOP_SENSING", 0xd2000018, 0, 0, 0, FALSE},

    {L"MFS1:", "MFS1", "MFS_READ_REGISTERS", 0xb0000004, INPUT_MFS_READ, sizeof(INPUT_MFS_READ), 1, TRUE},
    {L"MFS1:", "MFS1", "MFS_WRITE_REGISTERS", 0xb0000000, 0, 0, 0, FALSE},
    {L"MFS1:", "MFS1", "MFS_CONTROL_08", 0xb0000008, 0, 0, 0, FALSE},
    {L"MFS1:", "MFS1", "MFS_CONTROL_0C", 0xb000000c, 0, 0, 0, FALSE},
    {L"MFS1:", "MFS1", "MFS_CONTROL_10", 0xb0000010, 0, 0, 0, FALSE},

    {L"I2C2:", "I2C2", "I2C_READ", 0x80002005, INPUT_I2C_READ, sizeof(INPUT_I2C_READ), 1, TRUE},
    {L"I2C3:", "I2C3", "I2C_READ", 0x80002005, INPUT_I2C_READ, sizeof(INPUT_I2C_READ), 1, TRUE},
    {L"I2C4:", "I2C4", "I2C_READ", 0x80002005, INPUT_I2C_READ, sizeof(INPUT_I2C_READ), 1, TRUE},
    {L"I2C5:", "I2C5", "I2C_READ", 0x80002005, INPUT_I2C_READ, sizeof(INPUT_I2C_READ), 1, TRUE},
    {L"I2C2:", "I2C2", "I2C_WRITE", 0x80002004, 0, 0, 0, FALSE},
    {L"I2C2:", "I2C2", "I2C_WRITE_READ", 0x80002006, 0, 0, 0, FALSE},

    {L"SMB1:", "SMB1", "SMB_GET_IMAGE", 0x01012ef4, 0, 0, 64, TRUE},
    {L"SMB1:", "SMB1", "SMB_GET_OFFSET", 0x01012ef8, 0, 0, 3, TRUE},
    {L"SMB1:", "SMB1", "SMB_GET_RANGE", 0x01012f14, 0, 0, 1, TRUE},
    {L"SMB1:", "SMB1", "SMB_GET_MODE", 0x01012f1c, 0, 0, 1, TRUE},
    {L"SMB1:", "SMB1", "SMB_GET_BANDWIDTH", 0x01012f24, 0, 0, 1, TRUE},
    {L"SMB1:", "SMB1", "SMB_GET_WAKE_UP_PAUSE", 0x01012f2c, 0, 0, 1, TRUE},
    {L"SMB1:", "SMB1", "SMB_GET_LOW_G_THRESHOLD", 0x01012f34, 0, 0, 1, TRUE},
    {L"SMB1:", "SMB1", "SMB_GET_LOW_G_COUNTDOWN", 0x01012f3c, 0, 0, 1, TRUE},
    {L"SMB1:", "SMB1", "SMB_GET_HIGH_G_COUNTDOWN", 0x01012f44, 0, 0, 1, TRUE},
    {L"SMB1:", "SMB1", "SMB_GET_LOW_G_DURATION", 0x01012f4c, 0, 0, 1, TRUE},
    {L"SMB1:", "SMB1", "SMB_GET_HIGH_G_THRESHOLD", 0x01012f54, 0, 0, 1, TRUE},
    {L"SMB1:", "SMB1", "SMB_GET_HIGH_G_DURATION", 0x01012f5c, 0, 0, 1, TRUE},
    {L"SMB1:", "SMB1", "SMB_GET_ANY_MOTION_THRESHOLD", 0x01012f64, 0, 0, 1, TRUE},
    {L"SMB1:", "SMB1", "SMB_GET_ANY_MOTION_COUNT", 0x01012f6c, 0, 0, 1, TRUE},
    {L"SMB1:", "SMB1", "SMB_GET_INTERRUPT_MASK", 0x01012f74, 0, 0, 1, TRUE},
    {L"SMB1:", "SMB1", "SMB_READ_ACCEL_X", 0x01012f7c, 0, 0, 2, TRUE},
    {L"SMB1:", "SMB1", "SMB_READ_ACCEL_Y", 0x01012f80, 0, 0, 2, TRUE},
    {L"SMB1:", "SMB1", "SMB_READ_ACCEL_Z", 0x01012f84, 0, 0, 2, TRUE},
    {L"SMB1:", "SMB1", "SMB_READ_TEMPERATURE", 0x01012f88, 0, 0, 1, TRUE},
    {L"SMB1:", "SMB1", "SMB_READ_ACCEL_XYZT", 0x01012f8c, 0, 0, 6, TRUE},
    {L"SMB1:", "SMB1", "SMB_GET_INTERRUPT_STATUS", 0x01012f90, 0, 0, 1, TRUE},
    {L"SMB1:", "SMB1", "SMB_READ_REG", 0x01012fb4, INPUT_ZERO, sizeof(INPUT_ZERO), 1, TRUE},
    {L"SMB1:", "SMB1", "SMB_GET_LOW_G_HYST", 0x01012fc0, 0, 0, 1, TRUE},
    {L"SMB1:", "SMB1", "SMB_GET_HIGH_G_HYST", 0x01012fc8, 0, 0, 1, TRUE},
};

static void AppendChar(char c) {
    if (g_report_len + 1 < REPORT_CAPACITY) {
        g_report[g_report_len++] = c;
        g_report[g_report_len] = 0;
    }
}

static void Append(const char* text) {
    while (*text) {
        AppendChar(*text++);
    }
}

static void AppendHexNibble(DWORD value) {
    AppendChar((char)(value < 10 ? ('0' + value) : ('A' + value - 10)));
}

static void AppendHex8(DWORD value) {
    int shift;
    Append("0x");
    for (shift = 28; shift >= 0; shift -= 4) {
        AppendHexNibble((value >> shift) & 0x0f);
    }
}

static void AppendDec(DWORD value) {
    char digits[16];
    int count = 0;
    if (value == 0) {
        AppendChar('0');
        return;
    }
    while (value != 0 && count < (int)sizeof(digits)) {
        digits[count++] = (char)('0' + (value % 10));
        value /= 10;
    }
    while (count > 0) {
        AppendChar(digits[--count]);
    }
}

static void AppendBytes(const BYTE* bytes, DWORD count) {
    DWORD i;
    DWORD shown = count < 16 ? count : 16;
    for (i = 0; i < shown; ++i) {
        if (i != 0) {
            AppendChar(' ');
        }
        AppendHexNibble((bytes[i] >> 4) & 0x0f);
        AppendHexNibble(bytes[i] & 0x0f);
    }
    if (count > shown) {
        Append(" ...");
    }
}

static void AppendProbePrefix(const Probe* probe) {
    Append(probe->device_name);
    Append(" ");
    Append(probe->name);
    Append(" code=");
    AppendHex8(probe->code);
    Append(" ");
}

static void RunProbe(const Probe* probe, DWORD* ran, DWORD* ok_count, DWORD* fail_count, DWORD* skipped) {
    BYTE output[64];
    DWORD returned = 0;
    DWORD last_error = 0;
    BOOL ok;
    DWORD i;
    HANDLE device;

    AppendProbePrefix(probe);
    if (!probe->readonly) {
        Append("SKIP mutating/control\r\n");
        ++(*skipped);
        return;
    }

    for (i = 0; i < sizeof(output); ++i) {
        output[i] = 0xcc;
    }

    device = CreateFileW(probe->device, GENERIC_READ, 0, 0, OPEN_EXISTING, 0, 0);
    if (device == INVALID_HANDLE_VALUE) {
        last_error = GetLastError();
        Append("OPEN_FAIL last=");
        AppendHex8(last_error);
        Append("\r\n");
        ++(*fail_count);
        return;
    }

    ok = DeviceIoControl(
        device,
        probe->code,
        (LPVOID)probe->input,
        probe->input_len,
        output,
        probe->output_len,
        &returned,
        0);
    last_error = GetLastError();
    CloseHandle(device);

    ++(*ran);
    if (ok) {
        ++(*ok_count);
    } else {
        ++(*fail_count);
    }

    Append("ok=");
    AppendDec(ok ? 1 : 0);
    Append(" returned=");
    AppendDec(returned);
    Append(" last=");
    AppendHex8(last_error);
    Append(" out=");
    AppendBytes(output, returned);
    Append("\r\n");
}

static BOOL WriteReport(void) {
    DWORD written = 0;
    HANDLE file = CreateFileW(REPORT_PATH, GENERIC_WRITE, 0, 0, CREATE_ALWAYS, FILE_ATTRIBUTE_NORMAL, 0);
    if (file == INVALID_HANDLE_VALUE) {
        return FALSE;
    }
    WriteFile(file, g_report, g_report_len, &written, 0);
    CloseHandle(file);
    return written == g_report_len;
}

int WINAPI WinMain(HINSTANCE, HINSTANCE, LPWSTR, int) {
    DWORD i;
    DWORD ran = 0;
    DWORD ok_count = 0;
    DWORD fail_count = 0;
    DWORD skipped = 0;
    BOOL wrote_report;
    WCHAR message[256];

    Append("ioctl_discovery read-only probe\r\n");
    Append("report_path=\\\\SDMMC Disk\\ioctl_discovery.txt\r\n\r\n");

    for (i = 0; i < ARRAY_COUNT(PROBES); ++i) {
        RunProbe(&PROBES[i], &ran, &ok_count, &fail_count, &skipped);
    }

    wrote_report = WriteReport();
    wsprintfW(
        message,
        L"ioctl_discovery\r\nran=%u ok=%u failed=%u skipped=%u\r\nreport=%s",
        ran,
        ok_count,
        fail_count,
        skipped,
        wrote_report ? REPORT_PATH : L"(write failed)");
    MessageBoxW(0, message, L"ioctl_discovery", MB_OK);
    return 0;
}
