using System.Runtime.InteropServices;
using Ironpages.Interop;

namespace Ironpages;

/// <summary>Version information for the packaged Ironpages native library.</summary>
public static class IronpagesInfo
{
    /// <summary>The C ABI generation required by this managed package.</summary>
    public const uint RequiredAbiVersion = 1;

    /// <summary>Gets the C ABI generation implemented by the loaded native library.</summary>
    public static uint AbiVersion => NativeMethods.ironpages_abi_version();

    /// <summary>Gets the Ironpages package version from the loaded native library.</summary>
    public static string Version
    {
        get
        {
            var version = Marshal.PtrToStringUTF8(NativeMethods.ironpages_version());
            return version ?? throw new IronpagesException(
                IronpagesErrorKind.Internal,
                (int)NativeStatus.Internal,
                "The native library returned no package version.");
        }
    }

    internal static void EnsureCompatibleAbi()
    {
        var actual = AbiVersion;
        if (actual != RequiredAbiVersion)
        {
            throw new PlatformNotSupportedException(
                $"Ironpages ABI {RequiredAbiVersion} is required, but ABI {actual} was loaded.");
        }
    }
}
