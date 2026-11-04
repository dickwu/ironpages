using System.Runtime.InteropServices;

namespace Ironpages.Interop;

internal static class NativeResult
{
    internal static void EnsureSuccess(NativeStatus status, nint error)
    {
        using var errorOwner = ErrorHandle.Take(error);

        if (status == NativeStatus.Ok && errorOwner.IsInvalid)
        {
            return;
        }

        if (status == NativeStatus.Ok)
        {
            throw ContractViolation("A successful native call returned an error owner.");
        }

        var message = errorOwner.IsInvalid
            ? $"Ironpages failed with native status {(int)status}."
            : ReadErrorMessage(errorOwner);
        throw new IronpagesException(ToPublicKind(status), (int)status, message);
    }

    internal static ConverterHandle TakeConverter(
        NativeStatus status,
        nint converter,
        nint error)
    {
        var converterOwner = ConverterHandle.Take(converter);
        try
        {
            EnsureSuccess(status, error);
            if (converterOwner.IsInvalid)
            {
                throw ContractViolation("Native converter allocation returned no owner.");
            }

            return converterOwner;
        }
        catch
        {
            converterOwner.Dispose();
            throw;
        }
    }

    internal static byte[] TakePdf(NativeStatus status, nint pdf, nint error)
    {
        using var pdfOwner = BufferHandle.Take(pdf);
        EnsureSuccess(status, error);

        if (pdfOwner.IsInvalid)
        {
            throw ContractViolation("Native conversion returned no PDF owner.");
        }

        return CopyBytes(pdfOwner);
    }

    private static byte[] CopyBytes(BufferHandle buffer)
    {
        var length = NativeMethods.ironpages_buffer_len(buffer);
        if (length > int.MaxValue)
        {
            throw ContractViolation("Native PDF exceeds the managed array limit.");
        }

        var byteCount = (int)length;
        var data = NativeMethods.ironpages_buffer_data(buffer);
        if (byteCount > 0 && data == nint.Zero)
        {
            throw ContractViolation("Native PDF bytes are absent.");
        }

        var bytes = new byte[byteCount];
        if (byteCount > 0)
        {
            Marshal.Copy(data, bytes, 0, byteCount);
        }

        return bytes;
    }

    private static string ReadErrorMessage(ErrorHandle error)
    {
        var length = NativeMethods.ironpages_error_message_len(error);
        if (length > int.MaxValue)
        {
            return "Ironpages returned an oversized native diagnostic.";
        }

        var byteCount = (int)length;
        var data = NativeMethods.ironpages_error_message_data(error);
        if (byteCount == 0 || data == nint.Zero)
        {
            return "Ironpages returned an empty native diagnostic.";
        }

        return Marshal.PtrToStringUTF8(data, byteCount)
            ?? "Ironpages returned an invalid native diagnostic.";
    }

    private static IronpagesErrorKind ToPublicKind(NativeStatus status) => status switch
    {
        NativeStatus.InvalidArgument => IronpagesErrorKind.InvalidArgument,
        NativeStatus.InvalidUtf8 => IronpagesErrorKind.InvalidUtf8,
        NativeStatus.InvalidEnum => IronpagesErrorKind.InvalidEnum,
        NativeStatus.InvalidHandle => IronpagesErrorKind.InvalidHandle,
        NativeStatus.OutputNotEmpty => IronpagesErrorKind.OutputNotEmpty,
        NativeStatus.Parse => IronpagesErrorKind.Parse,
        NativeStatus.Css => IronpagesErrorKind.Css,
        NativeStatus.Layout => IronpagesErrorKind.Layout,
        NativeStatus.Render => IronpagesErrorKind.Render,
        NativeStatus.Font => IronpagesErrorKind.Font,
        NativeStatus.Io => IronpagesErrorKind.Io,
        NativeStatus.Security => IronpagesErrorKind.Security,
        NativeStatus.Internal => IronpagesErrorKind.Internal,
        _ => IronpagesErrorKind.Unknown,
    };

    private static IronpagesException ContractViolation(string message) =>
        new(IronpagesErrorKind.Internal, (int)NativeStatus.Internal, message);
}
