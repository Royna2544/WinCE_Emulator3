using System;
using System.Collections.ObjectModel;
using System.IO;
using System.Runtime.InteropServices.WindowsRuntime;
using System.Threading;
using System.Threading.Tasks;
using Microsoft.UI.Dispatching;
using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;
using Microsoft.UI.Xaml.Input;
using Microsoft.UI.Xaml.Media.Imaging;
using Windows.Foundation;
using Windows.Storage;
using Windows.Storage.Pickers;
using WinRT.Interop;

namespace CeRemote;

public sealed partial class MainWindow : Window
{
    private const int VncPort = 5900;
    private const int FilePort = 5001;

    private readonly DispatcherQueue _dispatcher;
    private readonly ObservableCollection<CeEntry> _entries = new();

    private VncClient? _vnc;
    private FileClient? _files;
    private WriteableBitmap? _bitmap;
    private CancellationTokenSource? _cts;
    private bool _pointerDown;

    public MainWindow()
    {
        InitializeComponent();
        _dispatcher = DispatcherQueue.GetForCurrentThread();
        FileList.ItemsSource = _entries;
        Title = "CE Remote";
    }

    private string Host => HostBox.Text.Trim();

    private void Log(string message) => _dispatcher.TryEnqueue(() => LogText.Text = message);

    private async void OnConnectClick(object sender, RoutedEventArgs e)
    {
        if (_vnc is not null)
        {
            Disconnect();
            return;
        }

        try
        {
            _cts = new CancellationTokenSource();
            _files = new FileClient(Host, FilePort);

            _vnc = new VncClient();
            _vnc.Disconnected += reason => _dispatcher.TryEnqueue(() =>
            {
                StatusText.Text = $"disconnected: {reason}";
                Disconnect();
            });
            _vnc.FrameReady += OnFrameReady;

            StatusText.Text = "connecting...";
            await _vnc.ConnectAsync(Host, VncPort, _cts.Token);

            _bitmap = new WriteableBitmap(_vnc.Width, _vnc.Height);
            ScreenImage.Source = _bitmap;
            StatusText.Text = $"connected {_vnc.Width}x{_vnc.Height}";
            ConnectButton.Content = "Disconnect";

            await RefreshListingAsync();
        }
        catch (Exception ex)
        {
            StatusText.Text = $"failed: {ex.Message}";
            Disconnect();
        }
    }

    private void Disconnect()
    {
        _cts?.Cancel();
        _vnc?.Dispose();
        _vnc = null;
        _files = null;
        _bitmap = null;
        ScreenImage.Source = null;
        ConnectButton.Content = "Connect";
    }

    private void OnFrameReady(byte[] frame)
    {
        _dispatcher.TryEnqueue(() =>
        {
            if (_bitmap is null)
            {
                return;
            }
            using var stream = _bitmap.PixelBuffer.AsStream();
            stream.Write(frame, 0, Math.Min(frame.Length, (int)_bitmap.PixelBuffer.Length));
            _bitmap.Invalidate();
        });
    }

    // ── VNC input ────────────────────────────────────────────────────────────

    private bool MapPointer(PointerRoutedEventArgs e, out int deviceX, out int deviceY)
    {
        deviceX = 0;
        deviceY = 0;
        if (_vnc is null || ScreenImage.ActualWidth <= 0 || ScreenImage.ActualHeight <= 0)
        {
            return false;
        }
        // The image is Stretch=Uniform: find the letterboxed content rect.
        double scale = Math.Min(ScreenImage.ActualWidth / _vnc.Width,
                                ScreenImage.ActualHeight / _vnc.Height);
        double contentW = _vnc.Width * scale;
        double contentH = _vnc.Height * scale;
        double offsetX = (ScreenImage.ActualWidth - contentW) / 2;
        double offsetY = (ScreenImage.ActualHeight - contentH) / 2;
        Point p = e.GetCurrentPoint(ScreenImage).Position;
        double localX = p.X - offsetX;
        double localY = p.Y - offsetY;
        if (localX < 0 || localY < 0 || localX >= contentW || localY >= contentH)
        {
            return false;
        }
        deviceX = (int)(localX / scale);
        deviceY = (int)(localY / scale);
        return true;
    }

    private async void OnScreenPointerPressed(object sender, PointerRoutedEventArgs e)
    {
        if (_vnc is null || _cts is null || !MapPointer(e, out int x, out int y))
        {
            return;
        }
        _pointerDown = true;
        ScreenImage.CapturePointer(e.Pointer);
        await _vnc.SendPointerAsync(true, x, y, _cts.Token);
    }

    private async void OnScreenPointerReleased(object sender, PointerRoutedEventArgs e)
    {
        if (_vnc is null || _cts is null || !MapPointer(e, out int x, out int y))
        {
            _pointerDown = false;
            return;
        }
        _pointerDown = false;
        await _vnc.SendPointerAsync(false, x, y, _cts.Token);
        ScreenImage.ReleasePointerCapture(e.Pointer);
    }

    private async void OnScreenPointerMoved(object sender, PointerRoutedEventArgs e)
    {
        if (!_pointerDown || _vnc is null || _cts is null || !MapPointer(e, out int x, out int y))
        {
            return;
        }
        await _vnc.SendPointerAsync(true, x, y, _cts.Token);
    }

    // ── File manager ─────────────────────────────────────────────────────────

    private async Task RefreshListingAsync()
    {
        if (_files is null || _cts is null)
        {
            return;
        }
        try
        {
            var listing = await _files.ListAsync(PathBox.Text.Trim(), _cts.Token);
            _entries.Clear();
            foreach (var entry in listing)
            {
                _entries.Add(entry);
            }
            Log($"{listing.Count} items in {PathBox.Text.Trim()}");
        }
        catch (Exception ex)
        {
            Log($"list failed: {ex.Message}");
        }
    }

    private async void OnRefreshClick(object sender, RoutedEventArgs e) => await RefreshListingAsync();

    private async void OnUpClick(object sender, RoutedEventArgs e)
    {
        PathBox.Text = FileClient.Parent(PathBox.Text.Trim());
        await RefreshListingAsync();
    }

    private async void OnPathKeyDown(object sender, KeyRoutedEventArgs e)
    {
        if (e.Key == Windows.System.VirtualKey.Enter)
        {
            await RefreshListingAsync();
        }
    }

    private async void OnFileDoubleTapped(object sender, DoubleTappedRoutedEventArgs e)
    {
        if (FileList.SelectedItem is not CeEntry entry)
        {
            return;
        }
        if (entry.IsDirectory)
        {
            PathBox.Text = FileClient.Join(PathBox.Text.Trim(), entry.Name);
            await RefreshListingAsync();
        }
        else
        {
            await DownloadSelectedAsync(entry);
        }
    }

    private async void OnDownloadClick(object sender, RoutedEventArgs e)
    {
        if (FileList.SelectedItem is CeEntry { IsDirectory: false } entry)
        {
            await DownloadSelectedAsync(entry);
        }
        else
        {
            Log("select a file to download");
        }
    }

    private async Task DownloadSelectedAsync(CeEntry entry)
    {
        if (_files is null || _cts is null)
        {
            return;
        }
        try
        {
            var picker = new FileSavePicker { SuggestedFileName = entry.Name };
            picker.FileTypeChoices.Add("All files", new[] { "." });
            InitializeWithWindow.Initialize(picker, WindowNative.GetWindowHandle(this));
            StorageFile? target = await picker.PickSaveFileAsync();
            if (target is null)
            {
                return;
            }
            string cePath = FileClient.Join(PathBox.Text.Trim(), entry.Name);
            byte[] data = await _files.DownloadAsync(cePath, _cts.Token);
            await FileIO.WriteBytesAsync(target, data);
            Log($"downloaded {entry.Name} ({data.Length} bytes)");
        }
        catch (Exception ex)
        {
            Log($"download failed: {ex.Message}");
        }
    }

    private async void OnUploadClick(object sender, RoutedEventArgs e)
    {
        if (_files is null || _cts is null)
        {
            return;
        }
        try
        {
            var picker = new FileOpenPicker();
            picker.FileTypeFilter.Add("*");
            InitializeWithWindow.Initialize(picker, WindowNative.GetWindowHandle(this));
            StorageFile? source = await picker.PickSingleFileAsync();
            if (source is null)
            {
                return;
            }
            var buffer = await FileIO.ReadBufferAsync(source);
            byte[] data = buffer.ToArray();
            string cePath = FileClient.Join(PathBox.Text.Trim(), source.Name);
            string reply = await _files.UploadAsync(cePath, data, _cts.Token);
            Log($"uploaded {source.Name}: {reply}");
            await RefreshListingAsync();
        }
        catch (Exception ex)
        {
            Log($"upload failed: {ex.Message}");
        }
    }
}
