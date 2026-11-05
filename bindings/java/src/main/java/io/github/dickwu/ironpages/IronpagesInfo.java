package io.github.dickwu.ironpages;

/** Version information for the packaged Ironpages native library. */
public final class IronpagesInfo {
  /** C ABI generation required by this Java package. */
  public static final int REQUIRED_ABI_VERSION = 1;

  /** Ironpages version carried by this Maven artifact. */
  public static final String PACKAGE_VERSION = "2.0.1";

  private IronpagesInfo() {}

  /** Return the C ABI generation implemented by the loaded native library. */
  public static int abiVersion() {
    return NativeLibraryLoader.api().ironpages_abi_version();
  }

  /** Return the Ironpages version implemented by the loaded native library. */
  public static String version() {
    return NativeLibraryLoader.nativeVersion();
  }
}
