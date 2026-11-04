using Ironpages.Interop;

namespace Ironpages;

/// <summary>A reusable, configured HTML and Markdown to PDF converter.</summary>
/// <remarks>
/// A converter owns one native handle. It may move between threads while idle,
/// but its methods and <see cref="Dispose"/> must not run concurrently.
/// </remarks>
public sealed partial class HtmlConverter : IDisposable
{
    private readonly ConverterHandle converter;

    /// <summary>Create a converter with safe defaults and no resource access.</summary>
    public HtmlConverter()
    {
        SupportedPlatform.EnsureCurrent();
        IronpagesInfo.EnsureCompatibleAbi();
        var status = NativeMethods.ironpages_converter_new(out var owner, out var error);
        converter = NativeResult.TakeConverter(status, owner, error);
    }

    /// <summary>Release the native converter deterministically.</summary>
    public void Dispose() => converter.Dispose();

    private void EnsureOpen() =>
        ObjectDisposedException.ThrowIf(converter.IsClosed, this);

    private HtmlConverter Complete(NativeStatus status, nint error)
    {
        NativeResult.EnsureSuccess(status, error);
        return this;
    }
}
