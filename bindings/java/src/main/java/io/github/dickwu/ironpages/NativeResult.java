package io.github.dickwu.ironpages;

import com.sun.jna.Pointer;
import com.sun.jna.ptr.PointerByReference;
import io.github.dickwu.ironpages.internal.SizeT;
import java.nio.charset.StandardCharsets;

/** Converts native owners and failures into Java values with one release path. */
final class NativeResult {
  private static final int OK = 0;
  private static final int INTERNAL = 255;

  private NativeResult() {}

  static Pointer takeConverter(
      NativeApi api, int status, PointerByReference converter, PointerByReference error) {
    if (status != OK || error.getValue() != null) {
      try {
        ensureSuccess(api, status, error);
      } finally {
        releaseConverter(api, converter);
      }
    }

    var owner = converter.getValue();
    if (owner == null) {
      throw contractViolation("Native converter allocation returned no owner.");
    }
    return owner;
  }

  static byte[] takePdf(
      NativeApi api, int status, PointerByReference pdf, PointerByReference error) {
    try {
      ensureSuccess(api, status, error);
      var owner = pdf.getValue();
      if (owner == null) {
        throw contractViolation("Native conversion returned no PDF owner.");
      }
      return copyBytes(api, owner);
    } finally {
      releaseBuffer(api, pdf);
    }
  }

  static void ensureSuccess(NativeApi api, int status, PointerByReference error) {
    var errorOwner = error.getValue();
    try {
      if (status == OK && errorOwner == null) {
        return;
      }
      if (status == OK) {
        throw contractViolation("A successful native call returned an error owner.");
      }

      var message =
          errorOwner == null
              ? "Ironpages failed with native status " + status + "."
              : readErrorMessage(api, errorOwner);
      throw new IronpagesException(IronpagesErrorKind.fromNative(status), status, message);
    } finally {
      releaseError(api, error);
    }
  }

  static void closeConverter(NativeApi api, Pointer converter) {
    var owner = new PointerByReference(converter);
    var status = api.ironpages_converter_free(owner);
    if (status != OK || owner.getValue() != null) {
      throw contractViolation("Native converter release failed with status " + status + ".");
    }
  }

  private static byte[] copyBytes(NativeApi api, Pointer buffer) {
    var length = managedLength(api.ironpages_buffer_len(buffer), "PDF");
    var data = api.ironpages_buffer_data(buffer);
    if (length > 0 && data == null) {
      throw contractViolation("Native PDF bytes are absent.");
    }
    return length == 0 ? new byte[0] : data.getByteArray(0, length);
  }

  private static String readErrorMessage(NativeApi api, Pointer error) {
    var length = managedLength(api.ironpages_error_message_len(error), "diagnostic");
    var data = api.ironpages_error_message_data(error);
    if (length == 0 || data == null) {
      return "Ironpages returned an empty native diagnostic.";
    }
    return new String(data.getByteArray(0, length), StandardCharsets.UTF_8);
  }

  private static int managedLength(SizeT length, String valueName) {
    var value = length.longValue();
    if (value < 0 || value > Integer.MAX_VALUE) {
      throw contractViolation("Native " + valueName + " exceeds the Java array limit.");
    }
    return (int) value;
  }

  private static void releaseConverter(NativeApi api, PointerByReference owner) {
    if (owner.getValue() != null) {
      api.ironpages_converter_free(owner);
    }
  }

  private static void releaseBuffer(NativeApi api, PointerByReference owner) {
    if (owner.getValue() != null) {
      api.ironpages_buffer_free(owner);
    }
  }

  private static void releaseError(NativeApi api, PointerByReference owner) {
    if (owner.getValue() != null) {
      api.ironpages_error_free(owner);
    }
  }

  private static IronpagesException contractViolation(String message) {
    return new IronpagesException(IronpagesErrorKind.INTERNAL, INTERNAL, message);
  }
}
