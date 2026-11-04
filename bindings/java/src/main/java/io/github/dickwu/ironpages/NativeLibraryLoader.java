package io.github.dickwu.ironpages;

import com.sun.jna.Library;
import com.sun.jna.Native;
import com.sun.jna.Platform;
import java.nio.charset.StandardCharsets;
import java.util.Map;
import java.util.Set;

/** Loads and verifies the packaged native library before it owns any state. */
final class NativeLibraryLoader {
  private static final Set<String> SUPPORTED_PREFIXES =
      Set.of("linux-x86-64", "linux-aarch64", "darwin-x86-64", "darwin-aarch64", "win32-x86-64");

  private NativeLibraryLoader() {}

  static NativeApi api() {
    return runtime().api();
  }

  static String nativeVersion() {
    return runtime().version();
  }

  private static LoadedNative runtime() {
    if (!SUPPORTED_PREFIXES.contains(Platform.RESOURCE_PREFIX)) {
      throw new UnsupportedOperationException(
          "Ironpages does not ship a native asset for " + Platform.RESOURCE_PREFIX + ".");
    }
    return RuntimeHolder.RUNTIME;
  }

  private static LoadedNative load() {
    var options =
        Map.<String, Object>of(
            Library.OPTION_CLASSLOADER, NativeLibraryLoader.class.getClassLoader());
    var api = Native.load("ironpages_ffi", NativeApi.class, options);
    var abiVersion = api.ironpages_abi_version();
    if (abiVersion != IronpagesInfo.REQUIRED_ABI_VERSION) {
      throw new LinkageError(
          "Ironpages ABI "
              + IronpagesInfo.REQUIRED_ABI_VERSION
              + " is required, but ABI "
              + abiVersion
              + " was loaded.");
    }

    var nativeVersion = readNativeVersion(api);
    if (!IronpagesInfo.PACKAGE_VERSION.equals(nativeVersion)) {
      throw new LinkageError(
          "Ironpages Java "
              + IronpagesInfo.PACKAGE_VERSION
              + " loaded native library "
              + nativeVersion
              + ".");
    }
    return new LoadedNative(api, nativeVersion);
  }

  private static String readNativeVersion(NativeApi api) {
    var version = api.ironpages_version();
    if (version == null) {
      throw new LinkageError("The Ironpages native library returned no package version.");
    }
    return version.getString(0, StandardCharsets.UTF_8.name());
  }

  private record LoadedNative(NativeApi api, String version) {}

  private static final class RuntimeHolder {
    private static final LoadedNative RUNTIME = load();

    private RuntimeHolder() {}
  }
}
