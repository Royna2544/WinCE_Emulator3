using System;
using System.Buffers.Binary;
using System.IO;
using System.Net.Sockets;
using System.Threading;
using System.Threading.Tasks;

namespace CeRemote;

/// <summary>
/// Minimal RFB 3.3 (VNC) client matching host_progs/rfb_server.cpp. Requests a
/// 32-bpp BGRA pixel format so decoded Raw rectangles drop straight into a
/// WriteableBitmap, and exposes pointer/key injection back to the device.
/// </summary>
public sealed class VncClient : IDisposable
{
    public int Width { get; private set; }
    public int Height { get; private set; }

    /// <summary>Raised with a freshly decoded full-frame BGRA buffer (Width*Height*4).</summary>
    public event Action<byte[]>? FrameReady;
    public event Action<string>? Disconnected;

    private TcpClient? _tcp;
    private NetworkStream? _stream;
    private byte[] _frame = Array.Empty<byte>();
    private CancellationTokenSource? _cts;
    private byte _buttons;
    private readonly SemaphoreSlim _sendLock = new(1, 1);

    public async Task ConnectAsync(string host, int port, CancellationToken token)
    {
        _tcp = new TcpClient { NoDelay = true };
        await _tcp.ConnectAsync(host, port, token).ConfigureAwait(false);
        _stream = _tcp.GetStream();

        await HandshakeAsync(token).ConfigureAwait(false);

        _frame = new byte[Width * Height * 4];
        _cts = CancellationTokenSource.CreateLinkedTokenSource(token);
        _ = Task.Run(() => ReceiveLoopAsync(_cts.Token));

        await RequestFrameAsync(false, token).ConfigureAwait(false);
    }

    private async Task HandshakeAsync(CancellationToken token)
    {
        var version = new byte[12];
        await ReadFullAsync(version, token).ConfigureAwait(false);
        await WriteAsync("RFB 003.003\n"u8.ToArray(), token).ConfigureAwait(false);

        // RFB 3.3: server sends a single u32 security type.
        var security = new byte[4];
        await ReadFullAsync(security, token).ConfigureAwait(false);
        uint securityType = BinaryPrimitives.ReadUInt32BigEndian(security);
        if (securityType == 0)
        {
            throw new IOException("server refused the connection");
        }
        // securityType 1 == None: no auth exchange follows.

        // ClientInit: shared-flag (keep other clients attached = 1).
        await WriteAsync(new byte[] { 1 }, token).ConfigureAwait(false);

        // ServerInit: width, height, 16-byte pixel format, name.
        var serverInit = new byte[24];
        await ReadFullAsync(serverInit, token).ConfigureAwait(false);
        Width = BinaryPrimitives.ReadUInt16BigEndian(serverInit.AsSpan(0));
        Height = BinaryPrimitives.ReadUInt16BigEndian(serverInit.AsSpan(2));
        uint nameLen = BinaryPrimitives.ReadUInt32BigEndian(serverInit.AsSpan(20));
        if (nameLen > 0)
        {
            await ReadFullAsync(new byte[nameLen], token).ConfigureAwait(false);
        }

        // Force 32-bpp little-endian BGRA so frames map onto a WriteableBitmap.
        await WriteAsync(BuildSetPixelFormat(), token).ConfigureAwait(false);
        // SetEncodings: Raw only (encoding 0).
        var encodings = new byte[]
        {
            2, 0, 0, 1, // type, padding, count = 1
            0, 0, 0, 0, // Raw
        };
        await WriteAsync(encodings, token).ConfigureAwait(false);
    }

    private static byte[] BuildSetPixelFormat()
    {
        var msg = new byte[20];
        msg[0] = 0; // SetPixelFormat
        msg[4] = 32; // bits-per-pixel
        msg[5] = 24; // depth
        msg[6] = 0; // big-endian flag
        msg[7] = 1; // true-colour
        BinaryPrimitives.WriteUInt16BigEndian(msg.AsSpan(8), 255); // red-max
        BinaryPrimitives.WriteUInt16BigEndian(msg.AsSpan(10), 255); // green-max
        BinaryPrimitives.WriteUInt16BigEndian(msg.AsSpan(12), 255); // blue-max
        msg[14] = 16; // red-shift
        msg[15] = 8; // green-shift
        msg[16] = 0; // blue-shift -> little-endian DWORD is B,G,R,A == BGRA
        return msg;
    }

    private async Task ReceiveLoopAsync(CancellationToken token)
    {
        try
        {
            var header = new byte[1];
            while (!token.IsCancellationRequested)
            {
                await ReadFullAsync(header, token).ConfigureAwait(false);
                switch (header[0])
                {
                    case 0:
                        await ReadFramebufferUpdateAsync(token).ConfigureAwait(false);
                        FrameReady?.Invoke(_frame);
                        await RequestFrameAsync(true, token).ConfigureAwait(false);
                        break;
                    case 1: // SetColourMapEntries
                        await SkipColourMapAsync(token).ConfigureAwait(false);
                        break;
                    case 2: // Bell
                        break;
                    case 3: // ServerCutText
                        await SkipServerCutTextAsync(token).ConfigureAwait(false);
                        break;
                    default:
                        throw new IOException($"unexpected server message {header[0]}");
                }
            }
        }
        catch (Exception ex) when (!token.IsCancellationRequested)
        {
            Disconnected?.Invoke(ex.Message);
        }
    }

    private async Task ReadFramebufferUpdateAsync(CancellationToken token)
    {
        var head = new byte[3];
        await ReadFullAsync(head, token).ConfigureAwait(false);
        int rects = BinaryPrimitives.ReadUInt16BigEndian(head.AsSpan(1));
        for (int i = 0; i < rects; i++)
        {
            var r = new byte[12];
            await ReadFullAsync(r, token).ConfigureAwait(false);
            int x = BinaryPrimitives.ReadUInt16BigEndian(r.AsSpan(0));
            int y = BinaryPrimitives.ReadUInt16BigEndian(r.AsSpan(2));
            int w = BinaryPrimitives.ReadUInt16BigEndian(r.AsSpan(4));
            int h = BinaryPrimitives.ReadUInt16BigEndian(r.AsSpan(6));
            int encoding = BinaryPrimitives.ReadInt32BigEndian(r.AsSpan(8));
            if (encoding != 0)
            {
                throw new IOException($"unsupported encoding {encoding}");
            }
            var rowBuffer = new byte[w * 4];
            for (int row = 0; row < h; row++)
            {
                await ReadFullAsync(rowBuffer, token).ConfigureAwait(false);
                int destY = y + row;
                if (destY < 0 || destY >= Height)
                {
                    continue;
                }
                int destOffset = (destY * Width + x) * 4;
                int copy = Math.Min(rowBuffer.Length, _frame.Length - destOffset);
                if (copy > 0)
                {
                    Array.Copy(rowBuffer, 0, _frame, destOffset, copy);
                }
            }
        }
    }

    private async Task SkipColourMapAsync(CancellationToken token)
    {
        var head = new byte[5];
        await ReadFullAsync(head, token).ConfigureAwait(false);
        int count = BinaryPrimitives.ReadUInt16BigEndian(head.AsSpan(3));
        if (count > 0)
        {
            await ReadFullAsync(new byte[count * 6], token).ConfigureAwait(false);
        }
    }

    private async Task SkipServerCutTextAsync(CancellationToken token)
    {
        var head = new byte[7];
        await ReadFullAsync(head, token).ConfigureAwait(false);
        uint length = BinaryPrimitives.ReadUInt32BigEndian(head.AsSpan(3));
        if (length > 0)
        {
            await ReadFullAsync(new byte[length], token).ConfigureAwait(false);
        }
    }

    public Task RequestFrameAsync(bool incremental, CancellationToken token)
    {
        var msg = new byte[10];
        msg[0] = 3; // FramebufferUpdateRequest
        msg[1] = (byte)(incremental ? 1 : 0);
        BinaryPrimitives.WriteUInt16BigEndian(msg.AsSpan(6), (ushort)Width);
        BinaryPrimitives.WriteUInt16BigEndian(msg.AsSpan(8), (ushort)Height);
        return WriteAsync(msg, token);
    }

    public Task SendPointerAsync(bool leftDown, int x, int y, CancellationToken token)
    {
        _buttons = (byte)(leftDown ? 1 : 0);
        var msg = new byte[6];
        msg[0] = 5; // PointerEvent
        msg[1] = _buttons;
        BinaryPrimitives.WriteUInt16BigEndian(msg.AsSpan(2), (ushort)Math.Clamp(x, 0, Width - 1));
        BinaryPrimitives.WriteUInt16BigEndian(msg.AsSpan(4), (ushort)Math.Clamp(y, 0, Height - 1));
        return WriteAsync(msg, token);
    }

    public async Task SendKeyAsync(uint keysym, bool down, CancellationToken token)
    {
        var msg = new byte[8];
        msg[0] = 4; // KeyEvent
        msg[1] = (byte)(down ? 1 : 0);
        BinaryPrimitives.WriteUInt32BigEndian(msg.AsSpan(4), keysym);
        await WriteAsync(msg, token).ConfigureAwait(false);
    }

    private async Task WriteAsync(byte[] data, CancellationToken token)
    {
        if (_stream is null)
        {
            return;
        }
        await _sendLock.WaitAsync(token).ConfigureAwait(false);
        try
        {
            await _stream.WriteAsync(data, token).ConfigureAwait(false);
            await _stream.FlushAsync(token).ConfigureAwait(false);
        }
        finally
        {
            _sendLock.Release();
        }
    }

    private async Task ReadFullAsync(byte[] buffer, CancellationToken token)
    {
        if (_stream is null)
        {
            throw new IOException("not connected");
        }
        int offset = 0;
        while (offset < buffer.Length)
        {
            int got = await _stream.ReadAsync(buffer.AsMemory(offset), token).ConfigureAwait(false);
            if (got <= 0)
            {
                throw new IOException("connection closed");
            }
            offset += got;
        }
    }

    public void Dispose()
    {
        _cts?.Cancel();
        _stream?.Dispose();
        _tcp?.Dispose();
        _sendLock.Dispose();
    }
}
