namespace Ironpages;

/// <summary>A categorized failure returned by the Ironpages renderer.</summary>
public sealed class IronpagesException : Exception
{
    internal IronpagesException(IronpagesErrorKind kind, int nativeStatus, string message)
        : base(message)
    {
        Kind = kind;
        NativeStatus = nativeStatus;
    }

    /// <summary>Gets the stable failure category.</summary>
    public IronpagesErrorKind Kind { get; }

    /// <summary>Gets the numeric status returned by the native ABI.</summary>
    public int NativeStatus { get; }
}
