package io.github.dickwu.ironpages;

/** Named physical page sizes understood by Ironpages. */
public enum PageSize {
  /** ISO A4, 210 by 297 millimetres. */
  A4(1),

  /** US Letter, 8.5 by 11 inches. */
  LETTER(2),

  /** US Legal, 8.5 by 14 inches. */
  LEGAL(3);

  private final int nativeValue;

  PageSize(int nativeValue) {
    this.nativeValue = nativeValue;
  }

  int nativeValue() {
    return nativeValue;
  }
}
