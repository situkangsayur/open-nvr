import 'package:flutter/material.dart';

/// Deep sea blue palette.
///
/// Dark is the default: a CCTV app is usually opened in a dim room, and a dark
/// surround makes the camera image itself the brightest thing on screen.
abstract final class NvrColors {
  static const primary = Color(0xFF0B3D91); // deep sea blue
  static const primaryDark = Color(0xFF08306B); // app bar, status bar
  static const accent = Color(0xFF00A8E8); // active controls, live badge
  static const background = Color(0xFF0A1929);
  static const surface = Color(0xFF12263A);
  static const surfaceRaised = Color(0xFF1B3550);
  static const border = Color(0xFF244665);
  static const textPrimary = Color(0xFFE8F1F8);
  static const textSecondary = Color(0xFF8FA9BF);
  static const online = Color(0xFF2ECC71);
  static const offline = Color(0xFFE74C3C);
  static const warning = Color(0xFFF39C12);
}

ThemeData buildNvrTheme() {
  const scheme = ColorScheme.dark(
    primary: NvrColors.accent,
    onPrimary: Color(0xFF00212E),
    secondary: NvrColors.accent,
    onSecondary: Color(0xFF00212E),
    surface: NvrColors.surface,
    onSurface: NvrColors.textPrimary,
    error: NvrColors.offline,
    onError: Colors.white,
  );

  return ThemeData(
    useMaterial3: true,
    brightness: Brightness.dark,
    colorScheme: scheme,
    scaffoldBackgroundColor: NvrColors.background,
    appBarTheme: const AppBarTheme(
      backgroundColor: NvrColors.primaryDark,
      foregroundColor: NvrColors.textPrimary,
      elevation: 0,
      centerTitle: false,
    ),
    cardTheme: CardThemeData(
      color: NvrColors.surface,
      elevation: 0,
      shape: RoundedRectangleBorder(
        borderRadius: BorderRadius.circular(12),
        side: const BorderSide(color: NvrColors.border),
      ),
      clipBehavior: Clip.antiAlias,
    ),
    inputDecorationTheme: InputDecorationTheme(
      filled: true,
      fillColor: NvrColors.surfaceRaised,
      labelStyle: const TextStyle(color: NvrColors.textSecondary),
      hintStyle: const TextStyle(color: NvrColors.textSecondary),
      border: OutlineInputBorder(
        borderRadius: BorderRadius.circular(10),
        borderSide: const BorderSide(color: NvrColors.border),
      ),
      enabledBorder: OutlineInputBorder(
        borderRadius: BorderRadius.circular(10),
        borderSide: const BorderSide(color: NvrColors.border),
      ),
      focusedBorder: OutlineInputBorder(
        borderRadius: BorderRadius.circular(10),
        borderSide: const BorderSide(color: NvrColors.accent, width: 2),
      ),
      isDense: true,
    ),
    filledButtonTheme: FilledButtonThemeData(
      style: FilledButton.styleFrom(
        backgroundColor: NvrColors.primary,
        foregroundColor: Colors.white,
        minimumSize: const Size.fromHeight(50),
        shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(10)),
        textStyle: const TextStyle(fontSize: 16, fontWeight: FontWeight.w600),
      ),
    ),
    outlinedButtonTheme: OutlinedButtonThemeData(
      style: OutlinedButton.styleFrom(
        foregroundColor: NvrColors.accent,
        side: const BorderSide(color: NvrColors.border),
        shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(10)),
      ),
    ),
    dividerTheme: const DividerThemeData(color: NvrColors.border, thickness: 1),
    progressIndicatorTheme: const ProgressIndicatorThemeData(color: NvrColors.accent),
    snackBarTheme: const SnackBarThemeData(
      backgroundColor: NvrColors.surfaceRaised,
      contentTextStyle: TextStyle(color: NvrColors.textPrimary),
      behavior: SnackBarBehavior.floating,
    ),
  );
}
