using System;
using System.Collections.Generic;
using System.Net.Http;
using System.Text;
using System.Threading;
using System.Threading.Tasks;

namespace CeRemote;

public sealed record CeEntry(bool IsDirectory, long Size, string Name)
{
    public string Glyph => IsDirectory ? "📁" : "📄";
    public string SizeText => IsDirectory ? "" : FormatSize(Size);

    private static string FormatSize(long size)
    {
        if (size < 1024)
        {
            return $"{size} B";
        }
        if (size < 1024 * 1024)
        {
            return $"{size / 1024.0:0.0} KB";
        }
        return $"{size / (1024.0 * 1024.0):0.0} MB";
    }
}

/// <summary>
/// Client for host_progs/mini_ftp.cpp on port 5001: list/download/upload over
/// HTTP with CE backslash paths URL-encoded into the <c>path</c> query value.
/// </summary>
public sealed class FileClient
{
    private readonly HttpClient _http = new() { Timeout = TimeSpan.FromSeconds(30) };
    private readonly string _host;
    private readonly int _port;

    public FileClient(string host, int port)
    {
        _host = host;
        _port = port;
    }

    private string Url(string verb, string cePath)
    {
        // Encode the whole CE path as one query value; the server URL-decodes it.
        string encoded = Uri.EscapeDataString(cePath);
        return $"http://{_host}:{_port}/{verb}?path={encoded}";
    }

    public async Task<List<CeEntry>> ListAsync(string cePath, CancellationToken token)
    {
        string text = await _http.GetStringAsync(Url("list", cePath), token).ConfigureAwait(false);
        var entries = new List<CeEntry>();
        foreach (string raw in text.Split('\n'))
        {
            string line = raw.TrimEnd('\r');
            if (line.Length < 2)
            {
                continue;
            }
            if (line[0] == 'D' && line[1] == ' ')
            {
                string name = line.Substring(2);
                if (name != "." && name != "..")
                {
                    entries.Add(new CeEntry(true, 0, name));
                }
            }
            else if (line[0] == 'F' && line[1] == ' ')
            {
                int space = line.IndexOf(' ', 2);
                if (space > 2 &&
                    long.TryParse(line.AsSpan(2, space - 2), out long size))
                {
                    entries.Add(new CeEntry(false, size, line.Substring(space + 1)));
                }
            }
        }
        return entries;
    }

    public async Task<byte[]> DownloadAsync(string cePath, CancellationToken token)
    {
        using var response =
            await _http.GetAsync(Url("download", cePath), token).ConfigureAwait(false);
        response.EnsureSuccessStatusCode();
        return await response.Content.ReadAsByteArrayAsync(token).ConfigureAwait(false);
    }

    public async Task<string> UploadAsync(string cePath, byte[] content, CancellationToken token)
    {
        using var body = new ByteArrayContent(content);
        using var response =
            await _http.PostAsync(Url("upload", cePath), body, token).ConfigureAwait(false);
        string reply = await response.Content.ReadAsStringAsync(token).ConfigureAwait(false);
        if (!response.IsSuccessStatusCode)
        {
            throw new HttpRequestException($"upload failed: {reply.Trim()}");
        }
        return reply.Trim();
    }

    /// <summary>Join a CE directory and a child name with a single backslash.</summary>
    public static string Join(string dir, string name)
    {
        if (dir.EndsWith('\\'))
        {
            return dir + name;
        }
        return dir + "\\" + name;
    }

    /// <summary>Return the parent of a CE path, or itself if already at a root.</summary>
    public static string Parent(string cePath)
    {
        string trimmed = cePath.TrimEnd('\\');
        int slash = trimmed.LastIndexOf('\\');
        if (slash <= 0)
        {
            return "\\";
        }
        return trimmed.Substring(0, slash);
    }
}
