package io.github.dickwu.ironpages;

/** A categorized failure returned by the Ironpages renderer. */
public final class IronpagesException extends RuntimeException {
  private static final long serialVersionUID = 1L;

  private final IronpagesErrorKind kind;
  private final int nativeStatus;

  IronpagesException(IronpagesErrorKind kind, int nativeStatus, String message) {
    super(message);
    this.kind = kind;
    this.nativeStatus = nativeStatus;
  }

  /** Return the stable failure category. */
  public IronpagesErrorKind getKind() {
    return kind;
  }

  /** Return the numeric status produced by the native ABI. */
  public int getNativeStatus() {
    return nativeStatus;
  }
}
