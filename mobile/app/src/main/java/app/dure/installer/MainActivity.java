package app.dure.installer;

import android.app.NativeActivity;

/**
 * Custom NativeActivity for Dure Installer.
 *
 * This Activity extends android.app.NativeActivity and uses fullscreen theme
 * for immersive edge-to-edge design. The Rust code queries window insets via JNI
 * and applies padding to the egui layout to prevent content from being obscured
 * by the status bar.
 */
public class MainActivity extends NativeActivity {
    // Native activity - no additional setup needed
    // Window insets are queried and applied in Rust via JNI
}
