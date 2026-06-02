#include <windows.h>
#include <mmsystem.h>
#include "../common/fixture_status.h"

#ifndef CALLBACK_EVENT
#define CALLBACK_EVENT 0x00050000
#endif
#ifndef WAVE_MAPPER
#define WAVE_MAPPER ((UINT)-1)
#endif
#ifndef WHDR_DONE
#define WHDR_DONE 0x00000001
#endif

int WINAPI WinMain(HINSTANCE, HINSTANCE, LPWSTR, int) {
    if (waveOutGetNumDevs() == 0) return FixtureFail(2201);

    WAVEFORMATEX fmt;
    ZeroMemory(&fmt, sizeof(fmt));
    fmt.wFormatTag = WAVE_FORMAT_PCM;
    fmt.nChannels = 1;
    fmt.nSamplesPerSec = 8000;
    fmt.wBitsPerSample = 16;
    fmt.nBlockAlign = 2;
    fmt.nAvgBytesPerSec = 16000;

    HANDLE done = CreateEventW(0, TRUE, FALSE, 0);
    if (!done) return FixtureFail(2202);

    HWAVEOUT out = 0;
    MMRESULT mm = waveOutOpen(&out, WAVE_MAPPER, &fmt, (DWORD)done, 0, CALLBACK_EVENT);
    if (mm != MMSYSERR_NOERROR || !out) return FixtureFail(2203);

    static short pcm[800];
    DWORD i;
    for (i = 0; i < 800; ++i) pcm[i] = ((i / 16) & 1) ? 5000 : -5000;

    WAVEHDR hdr;
    ZeroMemory(&hdr, sizeof(hdr));
    hdr.lpData = (LPSTR)pcm;
    hdr.dwBufferLength = sizeof(pcm);

    if (waveOutPrepareHeader(out, &hdr, sizeof(hdr)) != MMSYSERR_NOERROR) return FixtureFail(2204);
    if (waveOutWrite(out, &hdr, sizeof(hdr)) != MMSYSERR_NOERROR) return FixtureFail(2205);
    if (WaitForSingleObject(done, 5000) != WAIT_OBJECT_0) return FixtureFail(2206);
    if ((hdr.dwFlags & WHDR_DONE) == 0) return FixtureFail(2207);

    waveOutUnprepareHeader(out, &hdr, sizeof(hdr));
    waveOutSetVolume(out, 0x80008000);
    waveOutClose(out);
    CloseHandle(done);
    return FIXTURE_OK;
}
